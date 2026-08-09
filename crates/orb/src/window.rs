//! The writes over the two of the exe's imports the game's window is made by, and the black brush one of
//! those rewrites swaps in.
//!
//! `RegisterClassA` for the background that fills the letterbox, and `CreateWindowExA` for the arguments
//! the window is made with. Patching those entries catches the game's own window and nothing else's.
//!
//! **`register_class_a` is here rather than in [`orb_core::window`] with the other rewrite**, and it is
//! the one exception to where a hook body lives: the whole of what it does is one GDI call, and a seam
//! function for a call nothing else makes would be a worse tree than a body that stays beside the patch.
//! See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).
//!
//! Everything both rewrites decide is [`orb_core::window`]'s: where the window goes, where the game goes
//! inside it, the `Present` slot redirected into that rectangle, and the lines orb writes in the black
//! that is left.

use std::mem::offset_of;
use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::Rect;
use orb_config::Screen;
use windows_sys::Win32::Foundation::{COLORREF, RECT};
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    WNDCLASSA, WS_CAPTION, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_VISIBLE,
};

use orb_core::window::{BORDERLESS_STYLE, WINDOWED_STYLE, create_window_ex_a, is_game_class};

use crate::hook;

const BLACK: COLORREF = 0x0000_0000;

// The two styles, written out above the seam and held against Windows' own numbers here. A style is the
// game's own `CreateWindowExA` argument, so which one a window gets is decided where the rest of the
// layout is — and a number that drifted from `windows-sys` is a build that stops rather than a window
// nobody asked for.
const _: () = {
    assert!(BORDERLESS_STYLE == WS_POPUP | WS_VISIBLE);
    assert!(WINDOWED_STYLE == WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE);
};

// And `orb_api::Rect` held against `RECT` field by field, this being the last place in the tree that
// names `RECT` at all. `IDirect3DDevice8::Present`'s signature takes two of them and the replacement
// `orb_core::window` puts in that slot has to have that signature exactly, so the layout is what is being
// asserted: a size assert would pass whatever order the four came out in, which is the one thing worth
// asking.
const _: () = {
    assert!(offset_of!(Rect, left) == offset_of!(RECT, left));
    assert!(offset_of!(Rect, top) == offset_of!(RECT, top));
    assert!(offset_of!(Rect, right) == offset_of!(RECT, right));
    assert!(offset_of!(Rect, bottom) == offset_of!(RECT, bottom));
    assert!(size_of::<Rect>() == size_of::<RECT>());
};

static REGISTER_CLASS_A: AtomicUsize = AtomicUsize::new(0);

type RegisterClassA = unsafe extern "system" fn(*const WNDCLASSA) -> u16;

/// # Safety
/// Must run before the game creates its window, and `module` must be the exe.
pub unsafe fn install(
    module: usize,
    content: (u32, u32),
    screen: Screen,
) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "USER32.dll",
            "RegisterClassA",
            hook::address(register_class_a as _),
        )?
    };
    REGISTER_CLASS_A.store(previous, Ordering::Relaxed);
    // And the window's own arguments, whose rewrite is `orb-core`'s: what was in the entry is handed over
    // the way a laid-out game hands its own function over, and that handover is what settles the window
    // this game is to get.
    //
    // **Last, so that a launch which could not find one of the two entries settles nothing**: the ratio,
    // the screen and `SetProcessDPIAware` are what orb says because it is laying the window out, and a
    // launch whose imports are not there is not.
    let previous = unsafe {
        hook::install_import(
            module,
            "USER32.dll",
            "CreateWindowExA",
            hook::address(create_window_ex_a as _),
        )?
    };
    unsafe {
        orb_core::window::install_over(
            std::mem::transmute::<usize, orb_core::window::CreateWindowExA>(previous),
            content,
            screen,
        )
    };
    Ok(())
}

/// Gives the game's window class a black background, so the letterbox is painted
/// by Windows and repainted whenever the window is uncovered.
unsafe extern "system" fn register_class_a(class: *const WNDCLASSA) -> u16 {
    let original: RegisterClassA =
        unsafe { std::mem::transmute(REGISTER_CLASS_A.load(Ordering::Relaxed)) };
    if class.is_null() || !unsafe { is_game_class((*class).lpszClassName) } {
        return unsafe { original(class) };
    }
    let mut patched = unsafe { *class };
    patched.hbrBackground = unsafe { CreateSolidBrush(BLACK) } as HBRUSH;
    unsafe { original(&patched) }
}
