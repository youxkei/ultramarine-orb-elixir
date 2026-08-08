//! orb injected into the game's process, and nothing else.
//!
//! **What is here is what has no meaning without a process to patch**: `DllMain`, the trampolines and
//! the import table ([`hook`]), the PE headers those are read out of ([`pe`]), the crash handler that
//! names a module and an offset ([`crash`]), the six heap imports and the walk of what they hand out
//! ([`memtrack`]), the `CreateThread` import ([`threads`]), the two window imports and the `Present`
//! slot ([`window`]), the `CreateFileA` import ([`score`]), the `joyGetPosEx` entry and the thread that
//! samples it ([`joystick`]) — and the install lists below, which say which prologue goes with which
//! hook and which of them a `Config` asks for.
//!
//! **What a hook then does is [`orb_core::runtime`]**, and so is everything that decides what happens to
//! a run. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! Chapter-based retry for 東方紅魔郷 1.02h, injected before the game starts.

// The one thing the linker has to say about this crate, and it is not a fault: `DllMain` is
// `extern "system"`, so mingw's linker looks for the decorated `_DllMain@12`, finds the
// undecorated symbol Rust exported, and resolves the one to the other. That fixup is what makes
// the DLL's entry point work, and there is no spelling of the export that avoids it.
#![allow(linker_messages)]

// `pub` on the ones an `orb-e2e` scenario drives, which is what a crate outside this one needs of it: a
// game laid out by hand has no import table, so where a real launch patches an entry it calls the
// rewrite itself — `window::create_window_ex_a`, `score::create_file_a`, `joystick::answer` — and
// [`attach_to`] is how it is attached to with no process to patch. Nothing but those scenarios links this
// crate; the DLL exports `DllMain` and nothing else.
mod crash;
mod hook;
pub mod joystick;
pub mod memtrack;
mod pe;
pub mod score;
mod threads;
/// How much of the screen the game gets: the window created for it, and the black beside it.
///
/// `pub` for the same reason the hooks below are — a game laid out by hand calls
/// [`window::create_window_ex_a`] where the real game's own `CreateWindowExA` call lands, there being
/// no import table to reach it through — and for one thing besides: [`window::letterbox`] is where the
/// black either side of a 4:3 game is decided, and nothing else in orb says how much of it there is.
pub mod window;

use std::ffi::c_void;
use std::ops::Range;
use std::sync::atomic::Ordering;

use orb_config::Config;

// What has gone behind the seam, under the names the rest of this crate already calls it by.
// `log` carries both the module and the macro of that name — they are in different namespaces —
// which is why every call site can say `use crate::{detail, log}` whether it lives here or in
// `orb-core`, and go on saying it as it moves between them.
use orb_core::game::Game;
pub(crate) use orb_core::{detail, game, log, profile, resume, runtime, sync};
// What a hook *does* is `orb-core`'s, and so are the statics the install lists below fill: this crate
// installs them and calls none of them itself — see
// [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
use orb_core::runtime::{
    BLOCK_REPLAY_SAVE, CREATE_GAME_WINDOW, DECIDE_PRESSED, DECIDE_WAS_DOWN, FEED_DECIDE,
    FORCE_WINDOWED, GAME, GET_CONTROLLER_INPUT, GET_INPUT, HOLD_CANCEL, HOLD_DECIDE, IN_HOOK,
    INIT_D3D_DEVICE, INPUT_ACTIVE, INPUT_LOST, LAST_WORD, PACING, PLAY_SOUNDS, PRESENT,
    RANKING_READ, RENDER, RUN_CALC_CHAIN, RUN_CALC_CHAIN_TARGET, RUN_DRAW_CHAIN,
    RUN_DRAW_CHAIN_TARGET, SAVE_REPLAY, SETTLE_KEYS, STAGE_BEGUN, STAGE_BUILDING, STOP_RECORDING,
    TIME_JOYSTICK, UNLOCKS_READ, attached, create_game_window, get_controller_input, get_input,
    init_d3d_device, pacing, ranking_read, render, run_calc_chain, run_draw_chain, save_replay,
    stage_begun, stage_building, stop_recording, unlocks_read,
};
pub use orb_core::runtime::{Originals, detached};
use windows_sys::Win32::Foundation::{BOOL, HANDLE, TRUE};
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

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
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => log::close(),
        _ => {}
    }
    TRUE
}

/// What a hook body reaches back out to this crate for: the device's `Present` slot, and the lines
/// written in the black beside the game.
///
/// Both of them patch or draw over a real process, which is what this crate is and what `orb-core` is
/// not. Handed over as an install rather than compiled in, so that the same two arrive whether the game
/// is a process or a game laid out by hand.
fn patches() -> runtime::Patches {
    runtime::Patches {
        hook_device: window::hook_device,
        write_beside: window::write_beside,
    }
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

    // The two things a hook body does that need a process to do them to, handed the other way — see
    // `orb_core::runtime::Patches`.
    unsafe { runtime::hands_over_the_patches(patches()) };

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
    let Some(exe_path) = orb_api::module::host_exe() else {
        return log!("cannot locate the game exe; orb is doing nothing this run");
    };
    // Which game this is, before anything of the host is touched and before the config is read: a
    // process orb has no addresses for is one it must leave exactly as it found it, and every address
    // below this line belongs to one of these entries.
    let name = exe_path.file_name().unwrap_or_default().to_string_lossy();
    let Some(known) = game::known_by_exe(&name) else {
        return log!(
            "game: nothing orb knows is called {name}; it knows {}. orb is doing nothing this run",
            game::known_named(),
        );
    };
    log!(
        "game: {name}, and every address orb has for it was read off {}",
        known.version
    );
    // Stored before a single byte is patched, so that a hook this attach installs has a game to read
    // however the rest of the attach goes — see [`GAME`].
    unsafe { *GAME.get() = Some(known.game) };
    let game = known.game;
    unsafe { pacing() }.configure();

    let mut config = match Config::load_beside(&exe_path) {
        Ok(config) => config,
        Err(error) => return log!("config: {error}; orb is doing nothing this run"),
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

    let patches = game.hooks();
    // Remembered before the patches are consumed: orb's own frame loop calls the
    // game's chain functions at these addresses, which is what keeps its hooks in
    // the path.
    RUN_CALC_CHAIN_TARGET.store(patches.update.target, Ordering::Relaxed);
    RUN_DRAW_CHAIN_TARGET.store(patches.draw.target, Ordering::Relaxed);
    // And the two it calls in the game rather than reaching through a hook.
    let calls = game.frame_calls();
    PLAY_SOUNDS.store(calls.play_sounds);
    PRESENT.store(calls.present);

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
    match unsafe { joystick::install(exe, game.joystick_calibration()) } {
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
    match unsafe { window::install(exe, game.content_size(), config.screen) } {
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

    unsafe { attached(game, config, data) };
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
    // The same two as a real launch hands over, and for the same reason: a laid-out game's device has a
    // vtable to patch and its window has black beside it, so what a scenario drives is the whole of what
    // a hook does rather than a hook with two of its steps missing.
    unsafe { runtime::hands_over_the_patches(patches()) };
    log!(
        "orb {} attached to a game laid out in this process",
        env!("CARGO_PKG_VERSION")
    );
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    // Where [`attach`] settles this off the exe's own name, a game laid out by hand is handed over as
    // itself: there is no file to read the name of, and a scenario that had to name one would be
    // choosing its game twice.
    unsafe { *GAME.get() = Some(game) };
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
    // The two decisions [`attach`] takes by installing a hook or not — see [`BLOCK_REPLAY_SAVE`].
    BLOCK_REPLAY_SAVE.store(config.block_replay_save, Ordering::Relaxed);
    TIME_JOYSTICK.store(
        config.log_level >= orb_config::LogLevel::Verbose,
        Ordering::Relaxed,
    );
    for (slot, original) in [
        (&RUN_CALC_CHAIN, originals.update as usize),
        (&RUN_DRAW_CHAIN, originals.draw as usize),
        (&GET_INPUT, originals.input as usize),
        (&STAGE_BUILDING, originals.stage_building as usize),
        (&STAGE_BEGUN, originals.stage_begun as usize),
        (&UNLOCKS_READ, originals.unlocks_read as usize),
        (&RANKING_READ, originals.ranking_read as usize),
        (&RENDER, originals.render as usize),
        (&STOP_RECORDING, originals.stop_recording as usize),
        (&CREATE_GAME_WINDOW, originals.create_game_window as usize),
        (
            &GET_CONTROLLER_INPUT,
            originals.get_controller_input as usize,
        ),
        (&SAVE_REPLAY, originals.save_replay as usize),
        (&INIT_D3D_DEVICE, originals.init_d3d_device as usize),
        // What the patched call sites would be. In a real process these are the game's own two chain
        // functions with a jump to orb's hooks written over their prologues, so the frame loop calling
        // the address it was handed runs everything orb does per frame; here the hooks are what there
        // is, and calling them straight is the same path.
        (&RUN_CALC_CHAIN_TARGET, hook::address(run_calc_chain as _)),
        (&RUN_DRAW_CHAIN_TARGET, hook::address(run_draw_chain as _)),
    ] {
        slot.store(original, Ordering::Relaxed);
    }
    // And the two the frame loop calls in the game rather than reaching through a hook. Nothing to
    // call them on: see [`Originals`].
    PLAY_SOUNDS.store(game::Call {
        function: originals.play_sounds as usize,
        this: 0,
    });
    PRESENT.store(game::Call {
        function: originals.present as usize,
        this: 0,
    });
    // And the window, which is the same story: what the game gets is decided by orb either way, and a
    // laid-out game reaches the rewrite by calling it rather than by having its imports patched. Always,
    // as [`attach`] installs it always — the answer is orb's whichever of the two the settings say.
    unsafe { window::install_over(originals.create_window, game.content_size(), config.screen) };
    // And the display setting that window is made under, which is overruled once and where [`attach`]
    // overrules it: a game that has taken the display exclusively has no window to resize, and by the
    // time anything of orb's runs per frame the device already exists. Set here rather than in the reset
    // above because it is the one flag a fresh process brings *set*.
    FORCE_WINDOWED.store(true, Ordering::Relaxed);
    // And the joystick, which is the same story again: the read moves onto a thread of orb's own either
    // way, and what differs is whether the entry it stands in front of was patched or handed over.
    joystick::install_over(originals.joystick_position, game.joystick_calibration());
    log!("joystick: read on a thread of orb's, out of the game's frame");
    // The score file's fork, on the same gate [`attach`] puts it behind: only where a run can be rewound,
    // and always for a clear — there the fork is not what it is for, and being in the path of the write is.
    if config.chapters || config.fast_clear {
        if config.fast_clear {
            score::refuse_writes();
        }
        unsafe { score::install_over(originals.create_file) };
        if config.fast_clear {
            log!("score: no file is written this run");
        } else {
            log!("score: score.dat is forked while orb is in pointdevice mode");
        }
    }
    // The frame loop from nothing, and set up as [`attach`] sets it up: the cadence off the desktop's
    // own rate before there is a window to ask about, and the compositor's drawing time pinned where
    // the launch pinned it. Thrown away first rather than configured again, because what a `Pacing`
    // carries is a run's own — the frames it has counted and the gaps it has put them in are what the
    // `frame:` line is written from, and a game before this one's would be added to this one's.
    unsafe { *PACING.get() = None };
    unsafe { pacing() }.configure();
    unsafe { pacing() }.pin_compose(config.compose_us);
    unsafe { attached(game, config, data) };
}
