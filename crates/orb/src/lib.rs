//! Chapter-based retry for 東方紅魔郷 1.02h, injected before the game starts.

mod audio;
mod chapter;
mod crash;
mod frame;
mod d3d8;
mod game;
mod hook;
mod input;
mod log;
mod mem;
mod memtrack;
mod overlay;
mod pe;
mod profile;
mod retry_ui;
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
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use chapter::Chapters;
use game::th06::Th06;
use game::{Game, State};
use input::Keyboard;
use log::{log, summary};
use overlay::Overlay;
use retry_ui::{Choice, RetryMenu};
use snapshot::Snapshot;
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
/// Two minutes of game time — longer than any of the endings, and short enough that a
/// frame spent on the limit is a hitch rather than the several seconds a hundred
/// thousand updates took when the ending flag was wrongly set.
const ENDING_SKIP_LIMIT: u32 = 60 * 120;

/// How many times the stress mode restores the same chapter before letting the
/// run carry on. Without a limit it would rewind the first chapter for ever and
/// never reach a boss, which is where the paths worth exercising are.
const STRESS_PER_CHAPTER: u32 = 4;

struct Runtime {
    game: &'static dyn Game,
    config: Config,
    data: Range<usize>,
    frames: u32,
    previous: Option<State>,
    keyboard: Keyboard,
    /// F5/F9 slot, for checking the engine by hand.
    manual: Option<Snapshot>,
    /// Created on the first frame that has a Direct3D device; `None` for good
    /// once it has failed, so a broken overlay does not retry every frame.
    overlay: Option<Overlay>,
    overlay_ready: bool,
    /// The lag and the frame interval as the status line last showed them, in
    /// microseconds. Held between refreshes so the numbers can be read.
    shown: (i64, i64),
    chapters: Chapters,
    /// How far the play cursor has run past the next write, at its best and worst
    /// over a reporting interval. Judging the music by ear needs someone listening
    /// at the moment it slips; these numbers do not.
    margin_worst: u32,
    margin_best: u32,
    /// Frames left to report the margin every frame, set by a restore so that what
    /// a restore does to the streaming is visible rather than averaged away.
    margin_trace: u32,
    /// The stream the music was last seen playing through. The game replaces it
    /// when it changes track, which would leave a snapshot pointing at freed
    /// memory, so it is worth knowing exactly when that happens.
    stream: (usize, Option<u32>),
    /// Restores the stress mode has done in the chapter it is on.
    stressed: u32,
    stressing: (i32, u32),
    /// `Some` while the game is frozen on the retry menu.
    retry: Option<RetryMenu>,
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
/// Whether the game may read a joystick.
static JOYSTICK: AtomicBool = AtomicBool::new(true);

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

    log!("orb {} attached to pid {}", env!("CARGO_PKG_VERSION"), unsafe {
        GetCurrentProcessId()
    });

    let exe = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    log!("exe image base {exe:#010x}");
    let data = unsafe { pe::section(exe, b".data\0\0\0") };
    let Some(data) = data else {
        return log!("no .data section in the exe; orb is doing nothing this run");
    };
    log!(".data {:#010x}..{:#010x} ({} bytes)", data.start, data.end, data.len());
    frame::configure();

    let config = match log::host_exe().map(|path| Config::load_beside(&path)) {
        Some(Ok(config)) => config,
        Some(Err(error)) => return log!("config: {error}; orb is doing nothing this run"),
        None => return log!("cannot locate the game exe; orb is doing nothing this run"),
    };
    log!(
        "config: game_dir={} log_level={} self_check={} chapter_tuning={} \
         block_replay_save={} skip_ending={} borderless={} during_replay={} \
         stress_restore_frames={} chapters={} track_memory={} frame_hooks={}",
        config.game_dir.display(),
        config.log_level,
        config.self_check,
        config.chapter_tuning,
        config.block_replay_save,
        config.skip_ending,
        config.borderless,
        config.during_replay,
        config.stress_restore_frames,
        config.chapters,
        config.track_memory,
        config.frame_hooks,
    );
    // Set after the line that says what it is, so the log always states the level it
    // is then written at.
    log::set_level(config.log_level);

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

    let patches = GAME.hooks();
    // Remembered before the patches are consumed: orb's own frame loop calls the
    // game's chain functions at these addresses, which is what keeps its hooks in
    // the path.
    RUN_CALC_CHAIN_TARGET.store(patches.update.target, Ordering::Relaxed);
    RUN_DRAW_CHAIN_TARGET.store(patches.draw.target, Ordering::Relaxed);

    let mut hooks = Vec::new();
    if config.frame_hooks {
        hooks.push(("update", patches.update, run_calc_chain as usize, &RUN_CALC_CHAIN));
        hooks.push(("draw", patches.draw, run_draw_chain as usize, &RUN_DRAW_CHAIN));
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
    JOYSTICK.store(config.joystick, Ordering::Relaxed);
    match patches.joystick {
        // Hooked to skip the read, or just to time it when the log is being read
        // closely enough to want the split.
        Some(patch)
            if !config.joystick || config.log_level >= orb_config::LogLevel::Verbose =>
        {
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
    if config.borderless {
        match unsafe { window::install(exe, GAME.content_size()) } {
            Ok(()) => log!("borderless: window hooks installed"),
            Err(error) => log!("borderless: {error}; the window is left as the game makes it"),
        }
        if let Some(patch) = patches.create_window {
            FORCE_WINDOWED.store(true, Ordering::Relaxed);
            hooks.push(("window creation", patch, create_game_window as usize, &CREATE_GAME_WINDOW));
        }
        if let Some(patch) = patches.init_device {
            hooks.push(("device init", patch, init_d3d_device as usize, &INIT_D3D_DEVICE));
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

    let tuning = config.chapter_tuning.then(|| config.base_dir.join("chapters.rs"));
    let during_replay = config.during_replay;
    unsafe {
        *RUNTIME.get() = Some(Runtime {
            game: &GAME,
            config,
            data,
            frames: 0,
            previous: None,
            keyboard: Keyboard::new(),
            manual: None,
            overlay: None,
            overlay_ready: false,
            shown: (0, 0),
            chapters: Chapters::new(&GAME, tuning, during_replay),
            margin_worst: 0,
            margin_best: u32::MAX,
            margin_trace: 0,
            stream: (0, None),
            stressed: 0,
            stressing: (-1, 0),
            retry: None,
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

    // A replay can be run faster than it was recorded, with every frame still
    // drawn, which is what makes a pass over a full run quick without skipping any
    // of it.
    let replaying = runtime
        .previous
        .is_some_and(|previous| previous.replay && previous.playing && !previous.demo);
    let speed = if replaying { runtime.config.replay_speed } else { 1 };
    let chain = game.chain();
    let (update, draw) =
        (RUN_CALC_CHAIN_TARGET.load(Ordering::Relaxed), RUN_DRAW_CHAIN_TARGET.load(Ordering::Relaxed));
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
    frame::wait_for_slot(speed, window);
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
        log!("input: window {}", if active { "in front" } else { "behind, keys not read" });
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

/// The joystick half of the input read, which is a tail call inside the keyboard half
/// and so cannot be told apart from outside it — nor skipped from outside it, which is
/// the other reason this is hooked.
///
/// Skipping it is worth having because it is not cheap: measured here at nine
/// milliseconds a frame, more than half the budget at 120Hz, spent inside the game's
/// own retries at getting hold of a device that will not answer.
extern "C" fn get_controller_input(buttons: u32) -> u16 {
    if !JOYSTICK.load(Ordering::Relaxed) {
        // The keyboard's buttons, with nothing added: what the function does when there
        // is no joystick to read.
        return buttons as u16;
    }
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
        // The game's own frame setup is part of the chain being held back, so the
        // viewport it would have set has to be set here instead.
        let device = unsafe { runtime.game.d3d_device() };
        if !device.is_null() {
            unsafe { runtime.game.set_play_viewport(device) };
        }
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
    let fast_forward = runtime
        .previous
        .is_some_and(|previous| previous.replay && previous.playing && !previous.demo);
    let repeats = if fast_forward { runtime.config.replay_speed } else { 1 };

    let mut result = CHAIN_BREAK;
    let mut state = runtime.previous.unwrap_or(unsafe { runtime.game.read_state() });
    // Timed around orb's own work only: the game's update runs in between, and
    // counting it made orb look like it was costing milliseconds a frame.
    let mut started = profile::now();
    for _ in 0..repeats {
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
        if result == CHAIN_EXIT_SUCCESS || result == CHAIN_EXIT_ERROR {
            break;
        }
    }

    // Runs the ending out inside the frame it starts on, which is what keeps it
    // off the screen entirely: drawing happens once per frame, and by the next one
    // the game has already moved to the scene after the ending. Jumping the scene
    // instead would be simpler, but the ending is also where the game sets the
    // clear flag and enters the score, and those have to happen.
    // Never during a demo or a replay. Only a run someone played reaches an ending worth
    // skipping, and a demo that looked like one is what turned this into a hundred
    // thousand updates a frame.
    if runtime.config.skip_ending && state.in_ending && !state.demo && !state.replay {
        let scene = state.scene;
        let mut frames = 0;
        // Stops at the scene change as well as at the flag, so that whatever follows the
        // ending is reached and then left alone.
        while state.in_ending
            && state.scene == scene
            && frames < ENDING_SKIP_LIMIT
            && result != CHAIN_EXIT_SUCCESS
            && result != CHAIN_EXIT_ERROR
        {
            result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
            state = unsafe { runtime.game.read_state() };
            frames += 1;
        }
        log!("ending skipped, {frames} frames run, scene {scene} -> {}", state.scene);
    }

    // Only mid-run: between scenes the objects a snapshot reaches through are
    // half torn down.
    if state.in_game && runtime.config.chapters {
        if runtime.keyboard.pressed(runtime.config.save_state_key.0) {
            unsafe { save_manual(runtime) };
        }
        if runtime.keyboard.pressed(runtime.config.load_state_key.0) {
            unsafe { load_manual(runtime) };
        }
        tune(runtime, &state);
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
        && runtime.previous.is_some_and(|previous| state.deaths > previous.deaths);
    if died && runtime.config.chapters && runtime.chapters.can_retry() {
        log!("died in chapter {}", runtime.chapters.number());
        runtime.retry = Some(RetryMenu::new());
    }

    let scene_changed = runtime
        .previous
        .is_none_or(|previous| previous.scene != state.scene);
    let due = state.in_game && runtime.frames % STATE_LOG_INTERVAL == 0;
    if scene_changed || due {
        summary!(
            "f{} {state} clears={}",
            runtime.frames,
            unsafe { runtime.game.clears_back_buffer() },
        );
    }

    runtime.previous = Some(state);
    runtime.frames += 1;
    // Here rather than in the draw hook: this writes on the window itself, which is not
    // something to do in the middle of a scene the game is drawing into.
    unsafe { write_status(runtime) };
    unsafe {
        profile::record(profile::Phase::Update, started);
        if profile::frame() {
            summary!("{}", frame::report());
            summary!(
                "audio: behind {}..{} bytes",
                if runtime.margin_best == u32::MAX { 0 } else { runtime.margin_best },
                runtime.margin_worst,
            );
            runtime.margin_best = u32::MAX;
            runtime.margin_worst = 0;
        }
    }
    result
}

/// Each step logs before it starts: the log timestamps are the only way to see
/// which step is slow, and a step that never returns is otherwise invisible.
unsafe fn save_manual(runtime: &mut Runtime) {
    log!("save: collecting regions");
    memtrack::log_tracked();
    let regions = unsafe { memtrack::regions(runtime.data.clone()) };
    let bytes: usize = regions.iter().map(|region| region.len).sum();
    log!("save: {} regions, {bytes} bytes; capturing", regions.len());

    let audio = snapshot::Audio {
        policy: snapshot::Music::Rewind(runtime.game.music()),
        identity: runtime.game.music_identity(),
        state: runtime.game.audio_state(),
        thread: runtime.game.audio_thread(),
    };
    let snapshot = unsafe { Snapshot::capture(&regions, audio, runtime.config.self_check) };
    log!(
        "save: captured {} regions, {} bytes, music: {}",
        snapshot.regions(),
        snapshot.bytes(),
        snapshot.has_music(),
    );

    if runtime.config.self_check {
        log!("save: self_check restoring");
        unsafe { snapshot.restore(runtime.game.music_identity()) };
        log!("save: self_check comparing");
        report_self_check("save", unsafe { snapshot.check() });
    }
    runtime.manual = Some(snapshot);
    log!("save: done");
}

unsafe fn load_manual(runtime: &mut Runtime) {
    let Some(snapshot) = &runtime.manual else { return log!("load: nothing saved") };
    log!("load: restoring {} regions", snapshot.regions());
    unsafe { snapshot.restore(runtime.game.music_identity()) };
    log!("load: restored");
    if runtime.config.self_check {
        report_self_check("load", unsafe { snapshot.check() });
    }
}

/// Watches the streaming margin, which is what a listener hears as the music
/// breaking up once it runs out.
unsafe fn watch_music(runtime: &mut Runtime, state: &State) {
    if !state.playing {
        return;
    }
    let Some(music) = runtime.game.music() else { return };
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
    if interval == 0 || !state.playing || state.unsettled || state.paused {
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
/// in the wrong place.
fn tune(runtime: &mut Runtime, state: &State) {
    let config = &runtime.config;
    let add = runtime.keyboard.pressed(config.tuning_add_key.0);
    let remove = runtime.keyboard.pressed(config.tuning_remove_key.0);
    let write = runtime.keyboard.pressed(config.tuning_write_key.0);
    let Some(tuning) = runtime.chapters.tuning() else { return };

    if add {
        tuning.add(state);
    }
    if remove {
        tuning.remove_last(state);
    }
    if write {
        tuning.write(&GAME);
    }
}

fn report_self_check(what: &str, check: snapshot::SelfCheck) {
    log!(
        "{what} self_check: {} saved region(s) did not restore, {} untracked region(s) changed, \
         {} change(s) in the process heap",
        check.unrestored.len(),
        check.changed_untracked.len(),
        check.changed_in_process_heap,
    );
    for region in check.unrestored.iter().chain(&check.changed_untracked).take(32) {
        log!("{what}:   {:#010x}+{:#x}", region.base, region.len);
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
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else { return };
    if !runtime.overlay_ready {
        let device = unsafe { runtime.game.d3d_device() };
        if device.is_null() {
            return;
        }
        runtime.overlay_ready = true;
        runtime.overlay = unsafe {
            Overlay::new(device, &runtime.config.game_dir.join("font.ttf"), FONT_HEIGHT)
        };
        log!("overlay: {}", if runtime.overlay.is_some() { "ready" } else { "unavailable" });
    }
    let Some(overlay) = &runtime.overlay else { return };

    if let Some(menu) = &mut runtime.retry {
        let area = runtime.game.play_area();
        unsafe { menu.draw(overlay, area, runtime.chapters.number(), runtime.chapters.retries()) };
        return;
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
    let Some(state) = runtime.previous else { return };
    // The lag is what the pacing costs, and it is only meaningful next to the rate it
    // buys: a low number with an uneven rate is not an improvement.
    //
    // Held still between refreshes so the numbers can be read at all.
    if runtime.frames % HUD_NUMBER_INTERVAL == 0 {
        runtime.shown = frame::status();
    }
    let (lag, interval) = runtime.shown;
    // A line each, because the black is usually bars down the sides of a widescreen
    // monitor rather than a strip under the game, and a strip that narrow takes one
    // short line at a time.
    let mut lines = vec![
        format!("CH {:02}  RETRY {}", runtime.chapters.number(), runtime.chapters.retries()),
        format!("INPUT LAG {}.{}ms", lag / 1000, lag % 1000 / 100),
        format!("{}.{}fps", 1_000_000 / interval.max(1), 10_000_000 / interval.max(1) % 10),
    ];
    if let Some(tuning) = runtime.chapters.tuning() {
        lines.push(format!("SCRIPT {}", state.script_frames));
        lines.push(format!("MARKS {}", tuning.count(state.stage)));
    }
    unsafe { window::write_beside(&lines) };
}

