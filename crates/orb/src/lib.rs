//! orb injected into the game's process, and nothing else.
//!
//! **What is here is code whose subject is another module's memory**: `DllMain`, the jump written over a
//! prologue and the import table entry swapped ([`hook`]), the PE headers those are read out of ([`pe`]),
//! the crash handler that names a module and an offset when a fault happens in the process this was
//! injected into ([`crash`]), the six heap imports ([`memtrack`]), the `CreateThread` import
//! ([`threads`]), the two window imports and the black brush one of those rewrites swaps in ([`window`]),
//! the `CreateFileA` import ([`score`]), the `joyGetPosEx` entry ([`joystick`]) — and the install lists
//! below, which say which prologue goes with which hook and which of them a `Config` asks for.
//!
//! **What each of those entries is pointed at is [`orb_core`]'s**, and so is everything that decides what
//! happens to a run: every hook body, the rectangles the window is laid out by, the thread the pad is
//! sampled on, and `attach_to`. Nothing outside this crate names it — the DLL exports `DllMain` and
//! nothing else, and there is no `rlib` — which is what says so. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).
//!
//! Chapter-based retry for 東方紅魔郷 1.02h, injected before the game starts.

// The one thing the linker has to say about this crate, and it is not a fault: `DllMain` is
// `extern "system"`, so mingw's linker looks for the decorated `_DllMain@12`, finds the
// undecorated symbol Rust exported, and resolves the one to the other. That fixup is what makes
// the DLL's entry point work, and there is no spelling of the export that avoids it.
#![allow(linker_messages)]

// None of them `pub`: nothing outside this crate reaches any of it. A game laid out by hand has no import
// table, so where a real launch patches an entry it calls the rewrite itself — and every one of those
// rewrites is `orb-core`'s.
mod crash;
mod hook;
mod joystick;
mod memtrack;
mod mouse;
mod pe;
mod score;
mod threads;
mod window;

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use orb_config::Config;

// What has gone behind the seam, under the names the rest of this crate already calls it by.
// `log` carries both the module and the macro of that name — they are in different namespaces —
// which is why every call site can say `use crate::{detail, log}` whether it lives here or in
// `orb-core`, and go on saying it as it moves between them.
pub(crate) use orb_core::{game, log};
// What a hook *does* is `orb-core`'s, and so are the statics the install lists below fill: this crate
// installs them and calls none of them itself — see
// [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
use orb_core::runtime::{
    CREATE_GAME_WINDOW, FORCE_WINDOWED, GAME, GET_CONTROLLER_INPUT, GET_INPUT, INIT_D3D_DEVICE,
    PLAY_SOUNDS, PRESENT, RANKING_READ, RENDER, RUN_CALC_CHAIN, RUN_CALC_CHAIN_TARGET,
    RUN_DRAW_CHAIN, RUN_DRAW_CHAIN_TARGET, SAVE_REPLAY, STAGE_BEGUN, STAGE_BUILDING,
    STOP_RECORDING, UNLOCKS_READ, attached, create_game_window, get_controller_input, get_input,
    init_d3d_device, pacing, ranking_read, render, run_calc_chain, run_draw_chain, save_replay,
    stage_begun, stage_building, stop_recording, unlocks_read,
};
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
    let Some(exe_path) = orb_api::module::host_exe() else {
        return log!("cannot locate the game exe; orb is doing nothing this run");
    };
    // Which game this is, before anything of the host is touched and before the config is read: a
    // process orb has no addresses for is one it must leave exactly as it found it, and every address
    // below this line belongs to one of these entries.
    let name = exe_path.file_name().unwrap_or_default().to_string_lossy();
    let Some(known) = game::found(&name) else {
        return;
    };
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
            orb_core::score::refuse_writes();
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
    // And the pointer over that window, where the settings ask for it: a window is where Windows draws
    // one, and nothing in the game is played with the mouse. Left out rather than installed and idle
    // where they do not, the entry being worth patching only for a launch that is taking the pointer off.
    if config.hide_mouse {
        match unsafe { mouse::install(exe) } {
            Ok(()) => log!("mouse: the pointer goes while nothing is moving the mouse"),
            Err(error) => log!("mouse: {error}; the pointer is left as the game has it"),
        }
    } else {
        log!("mouse: hide_mouse is off; the pointer is left as the game has it");
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
