//! A mouse a test moves, and the display counter that decides whether its pointer is drawn.
//!
//! **The counter and not a flag, because that is what Windows keeps.** `ShowCursor` moves one number
//! by one step per call and the pointer is drawn while it is not negative, so two callers in a process
//! are two callers adding up — which is the whole of why orb cannot hide the pointer by asking once and
//! reading nothing back. The game asks too: 東方紅魔郷 1.02h answers `WM_SETCURSOR` with
//! `ShowCursor(TRUE)`, so a launch that let those calls through would be a launch where the counter
//! climbs by one for every mouse move over the window. Modelled here rather than smoothed over, so an
//! e2e test can hold orb against that.
//!
//! It starts at zero, which is what Windows starts it at where a mouse is installed.

use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

pub struct Mouse {
    at: Mutex<(i32, i32)>,
    /// The display counter, as `ShowCursor` keeps it — see [`Mouse::show`].
    count: AtomicI32,
    /// How many times it has been asked to move that counter, for an e2e test asking that orb asks
    /// once per change rather than once a frame.
    asks: AtomicU32,
    /// `GetCursorPos` failing, which is a host that will not say where the pointer is. Off by default:
    /// the ordinary case is one that answers, and a test that wants the other asks for it.
    refusing: Mutex<bool>,
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

impl Mouse {
    /// A pointer at the origin, drawn, and a host that will say where it is.
    pub fn new() -> Self {
        Self {
            at: Mutex::new((0, 0)),
            count: AtomicI32::new(0),
            asks: AtomicU32::new(0),
            refusing: Mutex::new(false),
        }
    }

    /// Moves the pointer, which is what somebody moving the mouse does.
    pub fn moves_to(&self, x: i32, y: i32) {
        *self.at.lock().unwrap() = (x, y);
    }

    /// Where it is, for a test that moves it from wherever it left it.
    pub fn at(&self) -> (i32, i32) {
        *self.at.lock().unwrap()
    }

    pub fn refuse(&self, refusing: bool) {
        *self.refusing.lock().unwrap() = refusing;
    }

    /// `GetCursorPos`: where the pointer is, or nothing at all from a host that will not say.
    pub fn position(&self) -> Option<(i32, i32)> {
        (!*self.refusing.lock().unwrap()).then(|| self.at())
    }

    /// `ShowCursor`: the counter moved one step, and what it reads afterwards.
    pub fn show(&self, showing: bool) -> i32 {
        self.asks.fetch_add(1, Ordering::Relaxed);
        let step = if showing { 1 } else { -1 };
        self.count.fetch_add(step, Ordering::Relaxed) + step
    }

    /// Whether the host is drawing the pointer, which is the counter not being negative.
    pub fn shown(&self) -> bool {
        self.count() >= 0
    }

    /// The counter itself, for an e2e test holding orb to one step either way: a launch that drove it
    /// to −7 is one whose next `ShowCursor(TRUE)` leaves the pointer where it was.
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }

    /// How many calls have been made, which is how an e2e test says orb asked on the change rather
    /// than every frame.
    pub fn asks(&self) -> u32 {
        self.asks.load(Ordering::Relaxed)
    }
}
