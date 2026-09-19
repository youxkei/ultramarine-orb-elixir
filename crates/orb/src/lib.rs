//! orb injected into the game's process, and nothing else.
//!
//! **What is here is code whose subject is another module's memory**: `DllMain`, the jump written over a
//! prologue and the import table entry swapped ([`hook`]), the PE headers those are read out of ([`pe`]),
//! the crash handler that names a module and an offset when a fault happens in the process this was
//! injected into ([`crash`]), the heap imports ([`memtrack`]), the `CreateThread` import
//! ([`threads`]), the two window imports and the black brush one of those rewrites swaps in ([`window`]),
//! the `CreateFileA` import ([`score`]), the `joyGetPosEx` entry ([`joystick`]) — and the install lists
//! below, which say which prologue goes with which hook and which of them a `Config` asks for.
//!
//! **What each of those entries is pointed at is [`orb_core`]'s**, and so is everything that decides what
//! happens to a run: every hook body, the rectangles the window is laid out by, the thread the pad is
//! sampled on, and `attach_to`. Nothing outside this crate names it — the DLL exports `DllMain` and
//! nothing else, and there is no `rlib` — which is what says so.
//!
//! Chapter-based retry for th06 1.02h and th07, injected before the game starts.

// The one thing the linker has to say about this crate, and it is not a fault: `DllMain` is
// `extern "system"`, so mingw's linker looks for the decorated `_DllMain@12`, finds the
// undecorated symbol Rust exported, and resolves the one to the other. That fixup is what makes
// the DLL's entry point work, and there is no spelling of the export that avoids it.
#![allow(linker_messages)]

// None of them `pub`: nothing outside this crate reaches any of it. A game laid out by hand has no import
// table, so where a real launch patches an entry it calls the rewrite itself — and every one of those
// rewrites is `orb-core`'s.
mod crash;
mod dialog;
mod hook;
mod joystick;
mod lookup;
mod memtrack;
mod mouse;
mod pe;
mod score;
mod steam;
mod threads;
mod window;

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use orb_config::Config;

// What has gone behind the seam, under the names the rest of this crate already calls it by.
// `log` carries both the module and the macro of that name — they are in different namespaces —
// which is why every call site can say `use crate::{detail, log}` whether it lives here or in
// `orb-core`, and go on saying it as it moves between them.
pub(crate) use orb_core::{game, log, runtime, trace};
// What a hook *does* is `orb-core`'s, and so are the statics the install lists below fill: this crate
// installs them and calls none of them itself.
use orb_core::runtime::{
    CREATE_GAME_WINDOW, FORCE_WINDOWED, GAME, GET_CONTROLLER_INPUT, GET_INPUT, INIT_D3D_DEVICE,
    PAD_BUTTONS, PLAY_SOUNDS, PRESENT, RANKING_READ, RENDER, RUN_CALC_CHAIN, RUN_CALC_CHAIN_TARGET,
    RUN_DRAW_CHAIN, RUN_DRAW_CHAIN_TARGET, SAVE_REPLAY, STAGE_BEGUN, STAGE_BUILDING,
    STOP_RECORDING, UNLOCKS_READ, attached, create_game_window, get_controller_input, get_input,
    init_d3d_device, pacing, pad_buttons, ranking_read, render, run_calc_chain, run_draw_chain,
    save_replay, stage_begun, stage_begun_in_ecx, stage_building, stop_recording, unlocks_read,
};
// **The four entry points only a 32-bit game reaches**, which are that architecture's in `orb-core` too:
// three of them stand in front of a callback the game's own chain calls with its argument in `ecx`, which
// is a convention x86 has and nothing else does, and the fourth is a `__thiscall`. No 64-bit game orb
// knows declares any of the four — see `orb_core::game::Hooks`, whose `joystick`, `unlocks_read`,
// `ranking_read` and `card_records_read` th06c and th06nc all answer `None` — so what the sites below do
// without them is say so rather than install one.
//
// **Unconditional, they are what kept `orb` from building for `x86_64-pc-windows-gnu` at all**, which is
// the target the 64-bit games' DLL is: `cargo xtask check` builds this crate for `TARGET` alone and its
// `ALSO` pass covers `orb-core` and `orb-e2e`, so nothing named the one build that was broken. See this
// change's own `TODO:` for the check that would have.
#[cfg(target_arch = "x86")]
use orb_core::runtime::{
    CARD_RECORDS_READ, card_records_read, get_controller_input_in_ecx, ranking_read_in_ecx,
    unlocks_read_in_ecx,
};
// The music's own two, which are the 32-bit games' — a `__thiscall` is an x86 calling convention and
// the hook that stands in front of one is that architecture's too. See `orb_core::runtime::Originals`.
#[cfg(target_arch = "x86")]
use orb_core::runtime::{STOPS_THE_MUSIC, TOPS_UP_THE_MUSIC, stops_the_music, tops_up_the_music};
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
        // **The trace's last chunk written here as well as the log's**, and before it: a launch the *game*
        // ended — the title's own last item, which is how somebody leaves one — is a launch orb was never
        // asked to close, so `close_the_game`'s own pair of closes never runs and the chunk in memory goes
        // with the process. Measured on `a_normal_mode_run_played_to_its_clear`, whose recipe walks to that
        // item: the trace ended ten frames past `step 12 of 16 done` and neither of the two steps after it
        // was in the file, where the log had this line to keep its own tail. And the trace first, because a
        // chunk that cannot be written says so through the log.
        DLL_PROCESS_DETACH => {
            trace::close();
            log::close();
        }
        _ => {}
    }
    TRUE
}

/// **The commit this was built from, which is what tells one build of a version from another.**
///
/// The version cannot: `0.11.0-dev` is what every build between two releases says, so two launches of two
/// builds open their logs with the same line. A pick-up driven against an exe deployed hours before a fix
/// read a file the way the build *without* that fix reads it and said so, and nothing in the log named the
/// build — the run that followed then wrote over the file the measurement was about.
///
/// `cargo xtask` puts it in the environment as it builds — see that crate's `built_from`, for what the id is
/// and why it is read from the working copy — and `option_env!` is what a build made some other way comes
/// through: it names no commit rather than a wrong one.
const BUILT_FROM: Option<&str> = option_env!("ORB_BUILT_FROM");

/// Runs with the game still suspended and the loader lock held: no loader
/// calls, no game code, just patching bytes and allocating our own memory.
fn attach() {
    log::open();
    std::panic::set_hook(Box::new(|info| log::line(&format!("panic: {info}"))));
    crash::install();

    log!(
        "orb {} attached to pid {}, {}",
        env!("CARGO_PKG_VERSION"),
        unsafe { GetCurrentProcessId() },
        BUILT_FROM.map_or_else(
            || "built outside cargo xtask, so no commit is named".to_owned(),
            |commit| format!("built from {commit}"),
        ),
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
    // And where its exe turned out to be, before a single address of it is read: a game built with ASLR
    // on is one whose addresses are written against a base the loader was free to refuse. See
    // `Game::set_image_base`.
    game.set_image_base(exe);
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
    // **The whole of it and not a line naming its fields**, which is what this was: every field appeared
    // twice — once in the format and once in the arguments — so a knob added to the config was a knob missing
    // from the line until somebody remembered both. `--pace` was added and left out, and the launch that ran
    // without it had nothing in its log to say so. Derived, the line cannot fall behind the struct.
    log!("config: {config:?}");
    // Set after the line that says what it is, so the log always states the level it
    // is then written at.
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    unsafe { pacing() }.pin_compose(config.compose_us);
    // And the frames asked for sooner than the game's own rate where the launch asked — see
    // `frame::Pacing::paces_faster`, which is what walking a run to some stage is spent on. Here beside the
    // rest of the pacing's own setup, and not in `attach_to`: that one is the seam a laid-out game attaches
    // through, so a launch of the real game never runs a line of it.
    unsafe { pacing() }.paces_faster(config.pace);

    let patches = game.hooks();
    // **And only where a run of this game can be rewound at all**, which is the whole of what the two
    // installations below are for: what they watch is the memory and the threads a chapter's snapshot has
    // to put back, and a game orb takes no snapshot of has nothing for them to watch. A `return` on the
    // failure is right for a game that does — a launch that would rewind against half a reading is worse
    // than one that does not start — and for th06c it ended the attach before the keyboard read was
    // hooked: that exe's `KERNEL32.dll` import list has no `HeapCreate` in it, so the install could not
    // find the entry it patched. Which entries a game's allocator and its threads go through is the
    // game's to say now — see `Game::rewinds`, `orb_core::game::Memory` and `orb_core::game::Threads`.
    if config.track_memory && game.rewinds() {
        let installed = match patches.takes_its_memory {
            game::Memory::Kernel32Imports => unsafe { memtrack::install(exe) },
            game::Memory::ProcessHeapImport => unsafe { memtrack::install_process_heap(exe) },
        };
        match installed {
            Ok(()) => {
                log!("memory hooks installed over {:?}", patches.takes_its_memory);
                // **Whether the walk of the process heap answers at all**, which is what says whether
                // anything orb decides off it means anything: `HeapWalk` refuses a heap with the
                // low-fragmentation heap on, and the process heap has it on by default. A guard read off
                // an empty answer is a guard that never fires, and `self_check`'s count of what changed
                // in this heap is the same reading.
                log!(
                    "memory: the process heap walks to {} region(s)",
                    orb_api::mem::process_heap_regions().len(),
                );
            }
            Err(error) => return log!("memory hooks: {error}; orb is doing nothing this run"),
        }
        let installed = match patches.makes_its_threads {
            game::Threads::CreateThread => unsafe { threads::install(exe) },
        };
        match installed {
            Ok(()) => log!("thread hook installed over {:?}", patches.makes_its_threads),
            Err(error) => return log!("thread hook: {error}; orb is doing nothing this run"),
        }
    }

    // Only where the score file might have to be somewhere other than the game's own: with
    // `--no-chapters` nothing can rewind, so every run belongs in the game's own ranking and
    // there is nothing to fork — and the same where the *game* has nothing orb can rewind, which is
    // what left a `pointdevice_score.dat` in a th07 install. See `Game::rewinds`. Loud rather than
    // fatal if the import is not there, since what it costs is which file scores land in and not a run
    // that cannot be played.
    //
    // A clear installs it whichever way that is set, because there the fork is not what it is
    // for: the file that must not be written is whichever this run would write, and refusing the
    // write means being in the path of it.
    if (config.chapters && game.rewinds()) || config.fast_clear {
        if config.fast_clear {
            orb_core::score::refuse_writes();
        }
        match unsafe { score::install(exe, patches.opens_its_files) } {
            Ok(()) if config.fast_clear => log!("score: no file is written this run"),
            Ok(()) => log!("score: score.dat is forked while orb is in pointdevice mode"),
            Err(error) => log!("score: {error}; the game will write its own score.dat"),
        }
    }

    // Remembered before the patches are consumed: orb's own frame loop calls the
    // game's chain functions at these addresses, which is what keeps its hooks in
    // the path.
    //
    // **Or its own two hooks, for a game that handed the walks over.** Nothing of that game's calls
    // either walk — the frame that did is orb's — so there is no prologue to be in the path of, and what
    // the loop calls is the hook with the handed-over walk behind it. The same wiring
    // `orb_core::runtime::attach_to` leaves for a game that is not a real process, and for the same
    // reason. See `orb_core::game::Walks`.
    match &patches.walks {
        Some(walks) => {
            RUN_CALC_CHAIN_TARGET.store(hook::address(run_calc_chain as _), Ordering::Relaxed);
            RUN_CALC_CHAIN.store(walks.update, Ordering::Relaxed);
            RUN_DRAW_CHAIN_TARGET.store(hook::address(run_draw_chain as _), Ordering::Relaxed);
            RUN_DRAW_CHAIN.store(walks.draw, Ordering::Relaxed);
        }
        None => {
            RUN_CALC_CHAIN_TARGET.store(
                patches.update.as_ref().map_or(0, |patch| patch.target),
                Ordering::Relaxed,
            );
            RUN_DRAW_CHAIN_TARGET.store(
                patches.draw.as_ref().map_or(0, |patch| patch.target),
                Ordering::Relaxed,
            );
        }
    }
    // **And whether what `--pace` speeds up is the game's own clock**, which is the case a game orb has
    // no frame hook in: `frame::Pacing` asks for the next frame sooner from inside the frame, and for a
    // game whose frames are its own there is nothing in there to shorten — th06c waits on a clock it
    // asks the host for, and a launch of it at `--pace=2` ran at the game's own rate before this. Taken
    // here because the patches are consumed below. See `orb_core::faster`.
    // **Through the frame where there is a frame to run and through the clocks where there is not.**
    // Running the game's own frame more than once is orb deciding how much of a run goes by in one of
    // them, which is pacing; answering the clocks moves the whole of it, the sound and the drawing with
    // it, and is what a game orb has no seam in the frame of has instead.
    // **And a frame loop of orb's own is a frame loop whichever way the game handed it over**, the two
    // walks being that as much as the two patches: `Pacing::paces_faster` above is what `--pace` does
    // there, and the game's own clock is not wanted on top of it. **Answering the clocks for such a game
    // is worse than useless**: the game then ticks twice for every word the rules are asked for, so half
    // of its frames are played blind, which is what two runs of th06c at `--pace=2` were doing while they
    // took twelve hits.
    let its_own_loop = patches.update.is_some() || patches.walks.is_some();
    let faster = config.pace > 1 && !its_own_loop;
    if faster {
        unsafe { orb_core::faster::runs_at(config.pace) };
    }
    // And the two it calls in the game rather than reaching through a hook.
    let calls = game.frame_calls();
    PLAY_SOUNDS.store(calls.play_sounds);
    PRESENT.store(calls.present);

    let mut hooks = Vec::new();
    // Where the game has them at all: a game orb has not been read far enough into to stand inside its
    // frame names neither, and what it gets is the hooks below that it does name — see
    // `orb_core::game::Hooks::update`.
    if let (true, Some(update), Some(draw)) = (config.frame_hooks, patches.update, patches.draw) {
        hooks.push((
            "update",
            update,
            hook::address(run_calc_chain as _),
            &RUN_CALC_CHAIN,
        ));
        hooks.push((
            "draw",
            draw,
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
    // The name the game asks for its keyboard read by, where it asks for one: taken here and installed
    // below with the window's, one entry answering both.
    let mut looked_up_read = None;
    match patches.input {
        // Two things want this one. Keeping the game updating with the window in the background
        // means the keys have to be dropped there, since the keyboard is read globally rather
        // than per window; and a run that can be picked up again is a run whose buttons are
        // written down here and handed back here.
        Some(game::Read::Code(patch)) if config.frame_hooks => {
            hooks.push(("input", patch, hook::address(get_input as _), &GET_INPUT));
        }
        // **And a pointer written where the game keeps its own read**, for a game that looked that
        // read up rather than calling it from its own code: no jump, no prologue, and what was
        // there is what orb calls through — see `orb_core::game::Read`.
        // **And the lookup of a read, for a game that asks for its own by name.** No code patched and
        // no address of the game's: orb answers the ask and the game writes orb's pointer into its own
        // table itself — see `orb_core::game::Read::LookedUp`, which has why the table is the wrong
        // place to write.
        Some(game::Read::LookedUp(looked_up)) if config.frame_hooks => {
            looked_up_read = Some(looked_up.name);
        }
        _ => {}
    }
    // **And the window's own making and moving, for a game that looks those up too** — one installation
    // for all of them, there being one `GetProcAddress` entry whatever is answered through it. Only where
    // the launch plays itself: what orb does with those two is put the window in a corner and hold it
    // there, and a window somebody plays in is the settings'. See
    // `orb_core::game::Hooks::makes_its_window`.
    let plays_itself = config.autoplay.is_some();
    let looked_up = runtime::Lookups {
        read: looked_up_read,
        // And the pad, which is how a run of a game like this is played at all — see
        // `orb_core::game::Hooks::reads_a_pad`. Only where the launch plays itself: a pad somebody is
        // holding is theirs, and orb answering the read would take it away from them.
        pad: patches
            .reads_a_pad
            .filter(|_| config.autoplay.is_some())
            .map(|looked_up| looked_up.name),
        window_made: patches
            .makes_its_window
            .filter(|_| plays_itself)
            .map(|looked_up| looked_up.name),
        window_moved: patches
            .moves_its_window
            .filter(|_| plays_itself)
            .map(|looked_up| looked_up.name),
        // And the two that put it on the screen, which go with the corner for the same reason: a window
        // kept hidden until its corner has held is a window whose put-backs nobody sees, and a launch
        // somebody plays has no such window to hide. See `orb_core::window::show_window`.
        window_shown: patches
            .shows_its_window
            .filter(|_| plays_itself)
            .map(|looked_up| looked_up.name),
        window_placed: patches
            .places_its_window
            .filter(|_| plays_itself)
            .map(|looked_up| looked_up.name),
        window_restyled: patches
            .restyles_its_window
            .filter(|_| plays_itself)
            .map(|looked_up| looked_up.name),
        // **And the clocks, for a game orb paces no frame of.** `--pace` shortens the wait between one
        // frame and the next from inside the frame, and a game orb is not inside waits on its own — so
        // what that number does here is scale the clock the game does the waiting by. Named without
        // asking the game, these being the host's own names: a game that imports its clock rather than
        // asking for it is one no answer of orb's reaches, and nothing is patched on its behalf. See
        // `orb_core::faster`.
        millis: faster.then_some("timeGetTime"),
        ticks: faster.then_some("GetTickCount"),
        counter: faster.then_some("QueryPerformanceCounter"),
        // **And the renderer, where the frame is orb's own**: what that answer is for is the present,
        // and the present is only orb's to make from inside a loop of orb's — see
        // `orb_core::game::Hooks::makes_its_renderer`.
        renderer: patches
            .makes_its_renderer
            .filter(|_| its_own_loop)
            .map(|looked_up| looked_up.name),
    };
    let names: Vec<&str> = [
        looked_up.read,
        looked_up.pad,
        looked_up.window_made,
        looked_up.window_moved,
        looked_up.window_shown,
        looked_up.window_placed,
        looked_up.window_restyled,
        looked_up.millis,
        looked_up.ticks,
        looked_up.counter,
        looked_up.renderer,
    ]
    .into_iter()
    .flatten()
    .collect();
    if !names.is_empty() {
        match unsafe { lookup::install(exe, looked_up) } {
            Ok(()) => log!(
                "hook: the game's own lookup of {} is answered by orb",
                names.join(", "),
            ),
            Err(error) => log!("hook: {error}; every name the game looks up is the host's own"),
        }
    }
    match patches.stage_begun {
        // Whatever mode the launch starts in, since the mode is answered per run: this is the
        // moment a stage's numbers are put in place, which is where they are read from and the
        // only place a resumed run's go back.
        // Entered the way the game's own callback is, which is not the same in the two games — see
        // `orb_core::game::Entered`.
        Some(entered) if config.chapters && config.resume => {
            let hook = match entered {
                game::Entered::OnTheStack(_) => hook::address(stage_begun as _),
                game::Entered::InEcx(_) => hook::address(stage_begun_in_ecx as _),
            };
            hooks.push((
                "stage start",
                game::Patch {
                    target: entered.patch().target,
                    prologue: entered.patch().prologue,
                },
                hook,
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
    // **Installed whatever the launch is for**, unlike the two above: what it says is what the game is
    // feeding its music buffer, which is as much a question of a launch with no chapters in it — and the
    // lines are at the verbose level, so a launch that is not being read pays a comparison per refill.
    #[cfg(target_arch = "x86")]
    if let Some(patch) = patches.tops_up_the_music {
        hooks.push((
            "music refill",
            patch,
            hook::address(tops_up_the_music as _),
            &TOPS_UP_THE_MUSIC,
        ));
    }
    // And the stop that ends it, for the same reason and at every launch: what a log has to be able to say is
    // where a silence began, and this is the one call that makes one.
    #[cfg(target_arch = "x86")]
    if let Some(patch) = patches.stops_the_music {
        hooks.push((
            "music stop",
            patch,
            hook::address(stops_the_music as _),
            &STOPS_THE_MUSIC,
        ));
    }
    // Loud rather than fatal: what it costs is the frame paying for the read again, which is
    // what every run before this did.
    match unsafe { joystick::install(exe) } {
        Ok(()) => log!("joystick: read on a thread of orb's, out of the game's frame"),
        Err(error) => log!("joystick: {error}; the read stays in the game's frame"),
    }
    if let Some(read) = &patches.joystick {
        // Always, and not called through: the pad half of the input read is orb's, for every pad the
        // machine has — see `orb_core::runtime::get_controller_input`. A launch that skipped it would be
        // one where no pad did anything at all.
        //
        // Which of the two entry points goes over it is the convention the game's own read has, and
        // nothing else about the two differs — see `orb_core::game::Entered`.
        let entry = match read {
            game::Entered::OnTheStack(_) => Some(hook::address(get_controller_input as _)),
            #[cfg(target_arch = "x86")]
            game::Entered::InEcx(_) => Some(hook::address(get_controller_input_in_ecx as _)),
            // **No entry point of that convention on this architecture**, which is what the import's own
            // comment says: the read is left the game's own and every pad the machine has reaches it the
            // game's way rather than orb's.
            #[cfg(not(target_arch = "x86"))]
            game::Entered::InEcx(_) => {
                log!(
                    "joystick: this game's read takes its argument in a register and no entry point \
                     here does; the read is left as the game has it"
                );
                None
            }
        };
        if let Some(entry) = entry {
            hooks.push((
                "joystick",
                game::Patch {
                    target: read.patch().target,
                    prologue: read.patch().prologue,
                },
                entry,
                &GET_CONTROLLER_INPUT,
            ));
        }
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
    match unsafe {
        window::install(
            exe,
            game.content_size(),
            config.screen,
            config.autoplay.is_some(),
        )
    } {
        Ok(()) => log!("screen: window hooks installed"),
        // The setting still goes in, for a game that sizes its own window off settings of its own and
        // can be told orb's — th06nc reads it on its first frame. See `orb_core::window::set_screen`.
        Err(error) => {
            unsafe { orb_core::window::set_screen(config.screen) };
            log!("screen: {error}; the window is left as the game makes it");
        }
    }
    // **Steam taken out of the launch**, for a game sold through it: it needs no client running to start,
    // puts no playtime on somebody's account, unlocks nothing and hands Steam no save to sync. **Whose run
    // it is decides where the file says nothing** — a run the machine played is not a run anybody played,
    // and one somebody is playing is theirs to have counted — which is `Config::without_steam_wanted`, and
    // what each import is answered with is `steam`.
    if config.without_steam_wanted() {
        match unsafe { steam::install(exe) } {
            Ok(()) => log!("steam: this launch reaches Steam not at all"),
            Err(error) => log!("steam: {error}; this launch is Steam's own"),
        }
    } else {
        log!("steam: this launch is Steam's own");
    }
    // **And whoever is playing this run, where a recipe was named.** The harness that plays it is its own
    // crate and this is the one place it is reached from — nothing of orb-core knows how a run is played, it
    // only keeps the slot. A launch nobody asked to play itself installs nothing and reads its keyboard.
    if let Some(recipe) = &config.autoplay {
        unsafe { orb_bot::install(recipe, game) };
        // And the dialog the game puts up before it starts anything, where it puts one up: a recipe's
        // first step cannot be pressed at a game that is still asking which window to make. Only for a
        // launch that plays itself — a person at the keyboard is who the dialog is *for* — which is what
        // puts it here rather than beside the rest of the installations.
        if let Some(answer) = patches.startup_dialog {
            match unsafe { dialog::install(exe, answer.clicks) } {
                Ok(()) => log!(
                    "dialog: the game's own startup dialog will be answered with {:?}",
                    answer.clicks,
                ),
                Err(error) => log!("dialog: {error}; the game's own startup dialog stands"),
            }
        }
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
        // Entered the way the game's own callback is, which differs between the two — see
        // `orb_core::game::Entered`.
        Some(entered) if config.chapters => {
            let bracket = match entered {
                game::Entered::OnTheStack(_) => Some(hook::address(unlocks_read as _)),
                #[cfg(target_arch = "x86")]
                game::Entered::InEcx(_) => Some(hook::address(unlocks_read_in_ecx as _)),
                // **No entry point of that convention on this architecture** — see the import's own
                // comment. A 64-bit game whose unlocks come out of such a callback is one whose front
                // end offers what the mode's own file says rather than what the game's does; th06c and
                // th06nc reach the read another way, `Game::reads_its_file_once_at_startup`.
                #[cfg(not(target_arch = "x86"))]
                game::Entered::InEcx(_) => {
                    log!(
                        "score: this game's unlocks read takes its argument in a register and no \
                         entry point here does; the read is left unbracketed"
                    );
                    None
                }
            };
            if let Some(bracket) = bracket {
                hooks.push((
                    "unlocks read",
                    game::Patch {
                        target: entered.patch().target,
                        prologue: entered.patch().prologue,
                    },
                    bracket,
                    &UNLOCKS_READ,
                ));
            }
        }
        _ => {}
    }
    // Always where the game has such a read, and what it does there is decided inside rather than here: the
    // record of captures is emptied only where there are two files to keep apart — see
    // `orb_core::runtime::ranking_read` — and what a session counted is put back inside a save whichever file
    // the mode points at, a legacy session's counts being its own as much as a rewound one's.
    // Entered the way the game's own callback is, which differs between the two — see
    // `orb_core::game::Entered`.
    if let Some(entered) = patches.ranking_read {
        let bracket = match entered {
            game::Entered::OnTheStack(_) => Some(hook::address(ranking_read as _)),
            #[cfg(target_arch = "x86")]
            game::Entered::InEcx(_) => Some(hook::address(ranking_read_in_ecx as _)),
            // **No entry point of that convention on this architecture** — see the import's own comment.
            #[cfg(not(target_arch = "x86"))]
            game::Entered::InEcx(_) => {
                log!(
                    "score: this game's ranking read takes its argument in a register and no entry \
                     point here does; the read is left unbracketed"
                );
                None
            }
        };
        if let Some(bracket) = bracket {
            hooks.push((
                "ranking read",
                game::Patch {
                    target: entered.patch().target,
                    prologue: entered.patch().prologue,
                },
                bracket,
                &RANKING_READ,
            ));
        }
    }
    // Always where the game has such a seam, and called through: what the file held stands, and what orb
    // adds is the counts of the run that has just ended — see `orb_core::runtime::card_records_read`. Not
    // conditional on the mode, a legacy run's counts belonging in the game's own file as much as a rewound
    // run's belong in orb's.
    // **32-bit alone**, the hook being a `__thiscall` — see the import's own comment, and
    // `orb_core::runtime::card_records_read`, which is that architecture's for the same reason. A 64-bit
    // game that declared this seam would be one to give the entry point an ABI of its own.
    #[cfg(target_arch = "x86")]
    if let Some(patch) = patches.card_records_read {
        hooks.push((
            "card records read",
            patch,
            hook::address(card_records_read as _),
            &CARD_RECORDS_READ,
        ));
    }
    #[cfg(not(target_arch = "x86"))]
    if patches.card_records_read.is_some() {
        log!(
            "score: this game's records read is a __thiscall and no entry point here is; the read is \
             left as the game has it"
        );
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
    // Always where the game has such a read, and called through: what a key config screen reads is the
    // pad's buttons by number, and a pad orb supplies is one the game has no device for — see
    // `orb_core::runtime::pad_buttons`.
    if let Some(patch) = patches.pad_buttons {
        hooks.push((
            "pad buttons",
            patch,
            hook::address(pad_buttons as _),
            &PAD_BUTTONS,
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
