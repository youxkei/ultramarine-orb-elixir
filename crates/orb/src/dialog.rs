//! The write over the one of the exe's imports the game puts its startup dialog up through, so that the
//! dialog is never made.
//!
//! `DialogBoxIndirectParamW`, which is how th06c asks which window it is to make before it makes one — a
//! template built in memory rather than a resource. Or `DialogBoxParamA` where the exe imports no such
//! entry, which is th11's: the same question out of the exe's own resources.
//!
//! **Answered, not made.** The call is answered with what the dialog's start button ends it with, and
//! nothing is created: no template is read, no window is made, no message loop runs. **And what the
//! dialog's procedure would have written is written here instead** — [`orb_core::game::Dialog`]'s
//! `settle` — because the game reads its answer out of the state it handed over: th06c's procedure writes
//! the mode it read and the vsync into `+8` and `+0xc` of it (`0x140026ad0`), and a launch that answered
//! without running the procedure at all was measured to end the game's own startup.
//!
//! **The first dialog and no more.** What was measured is the dialog the game puts up before it starts
//! anything; a second dialog later in a run is one nobody has read, and answering it blind would be
//! answering an error or a question about something else. Every one after the first is put up as the game
//! asks.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{HWND, LPARAM};

use crate::hook;
use crate::log;

/// The original of the exe's `DialogBoxIndirectParamW`, for every dialog after the first.
static DIALOG_BOX_INDIRECT_PARAM_W: AtomicUsize = AtomicUsize::new(0);
/// And of its `DialogBoxParamA`, for a game that puts its dialog up through that one.
static DIALOG_BOX_PARAM_A: AtomicUsize = AtomicUsize::new(0);
/// Whether the startup dialog has been answered already.
static ANSWERED: AtomicBool = AtomicBool::new(false);
/// **The game this launch is of**, whose own reading of its dialog's state the answer is settled with —
/// see `Game::settles_the_startup_dialog` — as the two words a `&dyn Game` is made of.
///
/// **Not a thread-local, which is what this was first**: the DLL's `DllMain` runs on the thread the
/// launcher injected it from, and the game's own startup — the dialog with it — runs later on the game's
/// own thread, so a per-thread answer is no answer at all. Two words because a trait object is a data
/// pointer and a table pointer, and neither is `Sync` as a type; the pair is written once, at the attach,
/// while the game's own thread is still suspended, and read after it is let go.
static THE_GAME: [AtomicUsize; 2] = [AtomicUsize::new(0), AtomicUsize::new(0)];

/// Says which game this launch is of, for [`startup_answered`].
fn the_game_is(game: &'static dyn crate::game::Game) {
    let words = unsafe { std::mem::transmute::<&'static dyn crate::game::Game, [usize; 2]>(game) };
    THE_GAME[0].store(words[0], Ordering::Release);
    THE_GAME[1].store(words[1], Ordering::Release);
}

/// The game said by [`the_game_is`], where one was.
fn the_game() -> Option<&'static dyn crate::game::Game> {
    let words = [
        THE_GAME[0].load(Ordering::Acquire),
        THE_GAME[1].load(Ordering::Acquire),
    ];
    if words[0] == 0 {
        return None;
    }
    Some(unsafe { std::mem::transmute::<[usize; 2], &'static dyn crate::game::Game>(words) })
}

/// What the start button ends a dialog with: `IDOK`.
const IDOK: isize = 1;

type DialogBoxIndirectParamW = unsafe extern "system" fn(
    *mut c_void,
    *const c_void,
    HWND,
    Option<DialogProc>,
    LPARAM,
) -> isize;

type DialogBoxParamA =
    unsafe extern "system" fn(*mut c_void, *const u8, HWND, Option<DialogProc>, LPARAM) -> isize;

type DialogProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

/// # Safety
/// `module` must be the exe, and this must run before the game has put its dialog up.
pub unsafe fn install(
    module: usize,
    game: &'static dyn crate::game::Game,
) -> Result<(), hook::Error> {
    the_game_is(game);
    match unsafe {
        hook::install_import(
            module,
            "USER32.dll",
            "DialogBoxIndirectParamW",
            hook::address(dialog_box_indirect_param_w as _),
        )
    } {
        Ok(previous) => DIALOG_BOX_INDIRECT_PARAM_W.store(previous, Ordering::Relaxed),
        // **Or th11's, a dialog out of the exe's own resources** — `DialogBoxParamA(hInstance, 0xcb, …)` at
        // 0x445626 — for an exe that imports no `DialogBoxIndirectParamW` at all.
        Err(hook::Error::ImportNotFound { .. }) => {
            let previous = unsafe {
                hook::install_import(
                    module,
                    "USER32.dll",
                    "DialogBoxParamA",
                    hook::address(dialog_box_param_a as _),
                )?
            };
            DIALOG_BOX_PARAM_A.store(previous, Ordering::Relaxed);
        }
        Err(error) => return Err(error),
    }
    Ok(())
}

/// The game's startup dialog, answered before it is made — and every dialog after it put up as the game
/// asked.
unsafe extern "system" fn dialog_box_indirect_param_w(
    instance: *mut c_void,
    template: *const c_void,
    parent: HWND,
    procedure: Option<DialogProc>,
    param: LPARAM,
) -> isize {
    if let Some(answer) = startup_answered(param) {
        return answer;
    }
    let original: DialogBoxIndirectParamW =
        unsafe { std::mem::transmute(DIALOG_BOX_INDIRECT_PARAM_W.load(Ordering::Relaxed)) };
    unsafe { original(instance, template, parent, procedure, param) }
}

/// The same, through the entry a game with the dialog in its resources puts one up with.
unsafe extern "system" fn dialog_box_param_a(
    instance: *mut c_void,
    name: *const u8,
    parent: HWND,
    procedure: Option<DialogProc>,
    param: LPARAM,
) -> isize {
    if let Some(answer) = startup_answered(param) {
        return answer;
    }
    let original: DialogBoxParamA =
        unsafe { std::mem::transmute(DIALOG_BOX_PARAM_A.load(Ordering::Relaxed)) };
    unsafe { original(instance, name, parent, procedure, param) }
}

/// The answer for the game's startup dialog, or `None` where that one has been answered already and this
/// dialog is another — with what the game's own procedure would have written settled first.
fn startup_answered(param: LPARAM) -> Option<isize> {
    if ANSWERED.swap(true, Ordering::Relaxed) {
        return None;
    }
    if let Some(game) = the_game() {
        unsafe { game.settles_the_startup_dialog(param as *mut c_void) };
    }
    log::line(&format!(
        "dialog: the game's own startup dialog is not made; it is answered with {IDOK}"
    ));
    Some(IDOK)
}
