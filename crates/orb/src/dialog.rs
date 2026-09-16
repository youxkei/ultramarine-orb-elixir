//! The write over the one of the exe's imports the game puts its startup dialog up through.
//!
//! `DialogBoxIndirectParamW`, which is how th06c asks which window it is to make before it makes one —
//! a template built in memory rather than a resource, and a dialog that comes up every launch whatever
//! is in the `th06.cfg` beside the exe.
//!
//! **What the rewrite does is hand the game's own dialog procedure a wrapper**, and the wrapper clicks
//! the controls a person would once the dialog is up. Which controls is
//! [`orb_core::game::Dialog`]'s — game knowledge, above the seam with the rest of it — and that only a
//! launch playing itself gets them clicked is `attach`'s.
//!
//! **The body is here rather than above the seam**, which is the one exception `orb::window`'s
//! `register_class_a` already is: the whole of what it does is forward to the procedure it was given and
//! post two messages, so a seam function for calls nothing else in the tree makes would be a worse tree
//! than a body that stays beside the patch.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BM_CLICK, GetDlgItem, PostMessageW, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
    WM_INITDIALOG,
};

use crate::hook;
use crate::log;

/// The dialog the game asked for, so the wrapper can call it.
static THE_GAMES_OWN: AtomicUsize = AtomicUsize::new(0);
/// And the original of the entry, which orb calls through with its own procedure in place of theirs.
static DIALOG_BOX_INDIRECT_PARAM_W: AtomicUsize = AtomicUsize::new(0);
/// Whether a dialog has been answered already.
///
/// **The first one and no more.** What was measured is the dialog the game puts up before it starts
/// anything, and a second dialog later in a run is one nobody has read: clicking a control by number in
/// a dialog orb knows nothing about is pressing whatever happens to carry that number.
static ANSWERED: AtomicBool = AtomicBool::new(false);

/// The controls to click, in order — see [`orb_core::game::Dialog`].
static CLICKS: std::sync::Mutex<&'static [i32]> = std::sync::Mutex::new(&[]);

type DialogBoxIndirectParamW = unsafe extern "system" fn(
    *mut c_void,
    *const c_void,
    HWND,
    Option<DialogProc>,
    LPARAM,
) -> isize;

type DialogProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize;

/// # Safety
/// `module` must be the exe, and this must run before the game has put its dialog up.
pub unsafe fn install(module: usize, clicks: &'static [i32]) -> Result<(), hook::Error> {
    *CLICKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = clicks;
    let previous = unsafe {
        hook::install_import(
            module,
            "USER32.dll",
            "DialogBoxIndirectParamW",
            hook::address(dialog_box_indirect_param_w as _),
        )?
    };
    DIALOG_BOX_INDIRECT_PARAM_W.store(previous, Ordering::Relaxed);
    Ok(())
}

/// The game's own dialog, put up with orb's procedure in front of the game's.
///
/// **The template, the parent and the parameter are the game's own and go through untouched**: what the
/// dialog *is* is the game's business, and the whole of orb's part in it is that the buttons get pressed.
unsafe extern "system" fn dialog_box_indirect_param_w(
    instance: *mut c_void,
    template: *const c_void,
    parent: HWND,
    procedure: Option<DialogProc>,
    param: LPARAM,
) -> isize {
    let original: DialogBoxIndirectParamW =
        unsafe { std::mem::transmute(DIALOG_BOX_INDIRECT_PARAM_W.load(Ordering::Relaxed)) };
    let Some(procedure) = procedure else {
        // A dialog with no procedure of its own is one there is nothing to forward to, and nothing orb
        // has anything to add to either.
        return unsafe { original(instance, template, parent, None, param) };
    };
    THE_GAMES_OWN.store(procedure as usize, Ordering::Relaxed);
    unsafe { original(instance, template, parent, Some(answered), param) }
}

/// Forwards every message to the game's own procedure, and once the dialog is up, posts the clicks.
///
/// **Posted and not sent, and after the game's own procedure has run.** `WM_INITDIALOG` is where the
/// dialog puts its own state on the controls — th06c ticks the radio button it opens on and the VSync
/// box — so a click sent from inside that message would be a click at a control the game is still
/// setting up. Posted, they are handled by the dialog's own loop in the order they went in, which is
/// what makes the last one the start button: it is the press the game reads the radio buttons on.
unsafe extern "system" fn answered(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    let theirs: DialogProc = unsafe { std::mem::transmute(THE_GAMES_OWN.load(Ordering::Relaxed)) };
    let answer = unsafe { theirs(window, message, wparam, lparam) };
    if message == WM_INITDIALOG && !ANSWERED.swap(true, Ordering::Relaxed) {
        // **The dialog goes in the same corner the window will**, which is what the first launch of this
        // looked wrong for: its template centres it, so the first thing a launch that plays itself put on
        // the screen was a box in the middle of somebody's monitor — measured at 1059,517 while the
        // window orb made was at the corner. Its own size is left alone; a dialog is a dialog.
        if let Some(corner) = orb_core::window::the_leftmost_of(&orb_api::window::monitors()) {
            unsafe {
                SetWindowPos(
                    window,
                    std::ptr::null_mut(),
                    corner.left,
                    corner.top,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                )
            };
        }
        let clicks = *CLICKS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for id in clicks {
            let control = unsafe { GetDlgItem(window, *id) };
            if control.is_null() {
                log::line(&format!(
                    "dialog: the game's own startup dialog has no control {id}; \
                     it is left to be answered by hand"
                ));
                return answer;
            }
            unsafe { PostMessageW(control, BM_CLICK, 0, 0) };
        }
        log::line(&format!(
            "dialog: the game's own startup dialog is answered by clicking {clicks:?}"
        ));
    }
    answer
}
