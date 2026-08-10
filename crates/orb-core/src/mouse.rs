//! The mouse pointer over the game's window: taken off the screen once the mouse has been still, put
//! back the moment it moves.
//!
//! Neither game is played with the mouse, and orb's answer to every display setting is a window — a
//! borderless one covering the monitor included, which is what leaves the pointer drawn where the game
//! taking the display exclusively would have had it hidden. So the pointer sits over the playfield until
//! somebody moves it away, and it is orb that has to take it off.
//!
//! **Windows draws the pointer by a display counter, and that counter is the whole process's.**
//! `ShowCursor` moves it one step per call and the pointer is drawn while it is not negative, so hiding
//! it means stepping over that edge and nothing else in the process may be stepping the other way. The
//! game is: **紅魔郷 1.02h answers `WM_SETCURSOR` itself** — its window procedure at 0x420d40 dispatches
//! `0x20` to the case at 0x420dc0 — and in a windowed launch, which is every launch of orb's, that case
//! is `LoadCursorA(IDC_ARROW)`, `SetCursor` and `ShowCursor(TRUE)`, answering 1 so `DefWindowProcA` never
//! sees the message. `WM_SETCURSOR` arrives whenever the pointer moves over the window, so a launch that
//! let those calls through would be a launch whose counter climbs by one per mouse move and never comes
//! down: one `ShowCursor(FALSE)` from orb would leave the pointer where it was, and the number of calls
//! it would take instead is the number of mouse moves since the window came up.
//!
//! **So orb takes the counter over.** The exe's `ShowCursor` import is patched to [`show_cursor`], which
//! answers the game rather than passing the call on, and the counter is then orb's alone — one step to
//! hide the pointer and one to put it back. Swallowing them all rather than only the ones that would
//! fight a pointer orb has hidden is the same argument: a call let through while the pointer is drawn
//! raises the counter without changing anything on the screen, and the step that comes later cannot
//! cross an edge that has moved.
//!
//! What is left in `orb::mouse` is the write over that import. Whether the pointer is on the screen is
//! decided here.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use orb_api::Hwnd;

use crate::detail;

/// How long the mouse is left alone before the pointer goes, in the milliseconds
/// [`orb_api::clock::ticks`] counts.
///
/// Three seconds. Nothing in either game wants the pointer, so the wait is only there to keep it from
/// being chased away between two movements of a hand that is still using it — long enough for that, and
/// short enough that it is gone before anybody has read it as part of the screen.
const STILL_FOR_MS: u32 = 3_000;

/// Whether orb is deciding this launch's pointer at all, which [`install`] is what settles.
///
/// Off is `hide_mouse: false`, and off as well where the patch over the exe's `ShowCursor` import did not
/// go in: there the counter is the game's as much as orb's, and a step orb takes is a step that hides
/// nothing — see the module's own note. So it leaves the pointer alone rather than fighting for it.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Whether the pointer is on the screen, as far as orb has asked.
///
/// Starts as drawn, which is what Windows starts a machine with a mouse at — and being the state orb
/// assumes rather than one it has asked for, the first call is only ever made on a change. Which is what
/// keeps orb off a machine with no mouse at all: Windows starts that one at −1, and a launch that pushed
/// the counter up to draw a pointer nobody has would be putting one on the screen.
static SHOWN: AtomicBool = AtomicBool::new(true);

/// What the host answered the last time orb moved the counter, which is what the game's own call is
/// answered with.
static COUNT: AtomicI32 = AtomicI32::new(0);

/// Says whether orb is deciding this launch's pointer, and puts one that a launch before it took off the
/// screen back.
///
/// `hiding` is `hide_mouse` out of `orb.yaml`. A launch that says no has nothing here to do, and does not
/// patch the entry either: what orb does about the pointer is own the host's counter, and there is nothing
/// to own where the pointer is not being taken off.
///
/// Called where the write over the exe's `ShowCursor` import went in — and by
/// [`crate::runtime::attach_to`] whichever way the setting is set, a laid-out game reaching the rewrite by
/// calling it rather than by having an import patched.
pub fn install(hiding: bool) {
    // The pointer put back rather than assumed to be there, which is the one thing a second launch in
    // one process would otherwise carry over from the first: orb's own step is the only one on that
    // counter, so a pointer the launch before this one left off the screen is orb's to undo. A launch
    // that is the first asks the host for nothing here.
    shown(true);
    ENABLED.store(hiding, Ordering::Relaxed);
}

/// What goes in that import entry: the game's ask, answered and not passed on.
///
/// The counter orb last read comes back, which is what the real call would have answered had orb's own
/// step been the last one. 紅魔郷 reads it at none of its six call sites.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// `ShowCursor` call lands, there being no import table to reach it through.
pub extern "system" fn show_cursor(_showing: i32) -> i32 {
    COUNT.load(Ordering::Relaxed)
}

/// The pointer as the frame loop follows it: where it was, and when it was last somewhere else.
pub struct Mouse {
    /// Where the pointer was on the last frame that could be told — `None` before the first of them.
    at: Option<(i32, i32)>,
    /// The stamp of that frame, in the milliseconds the log is stamped in.
    moved: u32,
}

/// Beside `new` because the module is public, and one with nothing read yet is exactly what `new` makes.
impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

impl Mouse {
    pub const fn new() -> Self {
        Self { at: None, moved: 0 }
    }

    /// Reads where the pointer is, and takes it off the screen once it has been in the same place for
    /// [`STILL_FOR_MS`] — while `ours` is the window in front, that being the pointer orb has anything
    /// to do with.
    ///
    /// **Read rather than waited for as a message.** `WM_MOUSEMOVE` and `WM_SETCURSOR` reach the window
    /// procedure, which is the game's, and standing in front of it for this would be a hook over the
    /// pump to learn what one read of the pointer says outright.
    ///
    /// **And `ours` for a different reason than the keyboard asks it for.** Keys read with another window
    /// in front are keys somebody typed at that window, and no movement of a mouse is the game's to act
    /// on at all. What this answers instead is how far a counter of this process reaches: the pointer over
    /// another program's window is not something orb has measured itself into, and one over a window that
    /// is not in front is in nobody's way — so a window going behind is answered by putting the pointer
    /// back and leaving the mouse to whatever is in front.
    pub fn poll(&mut self, ours: Hwnd) {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }
        let now = orb_api::clock::ticks();
        // Asked of the system rather than read from the game's own `WM_ACTIVATEAPP` flag, which only says
        // what the game was last told — the same question orb's keys are read behind, so the two cannot
        // disagree.
        if ours.is_null() || orb_api::window::foreground() != ours {
            // The wait from here, so that a window coming forward is three seconds of the mouse being
            // still and not the tail of a wait that ran while somebody was in another program.
            self.moved = now;
            shown(true);
            return;
        }
        // A host that will not say leaves the pointer as it is: one orb cannot follow is one it cannot
        // tell has stopped, and a pointer taken off the screen by a launch that then lost sight of the
        // mouse is one nothing puts back.
        let Some(at) = orb_api::mouse::position() else {
            return;
        };
        if self.at != Some(at) {
            self.at = Some(at);
            self.moved = now;
            shown(true);
            return;
        }
        // Wrapping, the stamp being the host's own count of milliseconds since it started: it comes
        // round every forty-nine days, and a difference worked out this way is right across that.
        if now.wrapping_sub(self.moved) >= STILL_FOR_MS {
            shown(false);
        }
    }
}

/// Asks the host for the pointer, on the frames the answer changes.
///
/// Once per change rather than once a frame, which is the whole of what the counter makes possible to
/// get wrong: every call moves it, so a launch that asked every frame would be one whose pointer is
/// hundreds of steps from coming back.
fn shown(showing: bool) {
    if SHOWN.swap(showing, Ordering::Relaxed) == showing {
        return;
    }
    let count = orb_api::mouse::show(showing);
    COUNT.store(count, Ordering::Relaxed);
    detail!(
        "mouse: the pointer is {} the screen, display counter {count}",
        if showing { "back on" } else { "off" },
    );
}
