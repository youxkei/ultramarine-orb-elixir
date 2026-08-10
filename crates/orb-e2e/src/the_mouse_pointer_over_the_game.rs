//! **The mouse pointer over the game: gone once the mouse has been still, back the moment it moves.**
//!
//! Neither game is played with the mouse and orb puts both of them in a window — a borderless one over
//! the whole monitor included — which is the launch Windows draws a pointer in. So the pointer sits over
//! the playfield for as long as nobody moves it, and taking it off the screen is orb's to do.
//!
//! **The display counter is one number the whole process shares.** `ShowCursor` moves it a step per call
//! and Windows draws the pointer while it is not negative, which is why the counter is what these read
//! back rather than a flag: a launch that drove it to −7 is one whose next `ShowCursor(TRUE)` leaves the
//! screen as it was. `orb_sim::Mouse` keeps it the way Windows keeps it, starting at the zero a machine
//! with a mouse starts at.
//!
//! **What orb takes off is the pointer over the game**, so which window the host has in front is half of
//! what these declare — the other half being where the pointer is and when it was moved there.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, WINDOW, in_its_own_process};
use orb_api::Hwnd;
use orb_sim::Clock;

/// Another program's window in front. Any handle that is not the game's: what makes it another window is
/// that the host names it as the one in front and not this one.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How long the mouse is left alone before orb takes the pointer off the screen, in microseconds.
///
/// Written out here rather than read off `orb_core::mouse`, which keeps it: what an e2e test holds is
/// the wait somebody sitting at the game gets, and a test that took its number from the code under test
/// would agree with it whatever either of them said.
const STILL_FOR_US: i64 = 3_000_000;

/// How far short of that wait a frame is run, to hold the half of the claim that says the pointer is
/// still there. A tenth of a second, which is six frames at the rate the game was written for — far
/// enough short that no rounding of a frame reaches over it.
const SHORT_OF_IT_US: i64 = 100_000;

/// How many frames the pointer is waited for after that, which is [`SHORT_OF_IT_US`] of them and room
/// over.
const FRAMES: u32 = 16;

/// A launch with the game's window made, which is the window a pointer is drawn over.
fn launched(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |_| {});
    game.creates_its_window();
    game
}

/// How long the mouse has been still, as the host's own clock has it.
fn still_for(game: &Fake, since: i64) -> i64 {
    Clock::micros_for_ticks(game.sim().clock().peek() - since)
}

/// The pointer goes once the mouse has been still for three seconds, comes back on the frame after it
/// moves, and goes again three seconds after *that* — the wait being from the last movement rather than
/// from the launch.
#[test]
fn the_pointer_goes_once_the_mouse_has_been_still_and_comes_back_when_it_moves() {
    in_its_own_process(|| {
        let game = launched("the-mouse-pointer");
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen on the launch's own first frame",
        );

        // The mouse moved once, so what follows is a wait from a moment this e2e test knows rather than
        // from whenever the launch's first frame read the pointer.
        game.sim().mouse().moves_to(700, 400);
        let since = game.sim().clock().peek();
        game.frame();

        // Three seconds of the host's clock, less a tenth, with nothing moving the mouse. Said as a
        // jump rather than as a hundred and eighty frames because the wait is the clock's: the same
        // three seconds on a display of any rate.
        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the pointer went after {}µs of the mouse being still",
            still_for(&game, since),
        );

        game.frames_until("the pointer taken off the screen", FRAMES, || {
            !game.sim().mouse().shown()
        });
        let waited = still_for(&game, since);
        assert!(
            waited >= STILL_FOR_US,
            "the pointer went after {waited}µs, which is short of the three seconds",
        );
        // The pointer is read once a frame, so it goes on the first frame past the wait and not inside
        // one: what that costs is a frame either side of the number, and nothing more.
        assert!(
            waited < STILL_FOR_US + 2 * game.refresh_period_us(),
            "the pointer went {}µs late, which is more than a frame past the wait",
            waited - STILL_FOR_US,
        );

        // One step of the counter and no more, which is what leaves the pointer one call from coming
        // back.
        assert_eq!(
            game.sim().mouse().count(),
            -1,
            "orb drove the display counter further than the one step that hides the pointer",
        );

        // And asked once, on the frame it changed: a call a frame would be a call into the host on
        // every frame of a run where nothing about the pointer has moved.
        let asked = game.sim().mouse().asks();
        game.frames(FRAMES);
        assert_eq!(
            game.sim().mouse().asks(),
            asked,
            "orb asked the host again with the pointer already off the screen",
        );

        // Back on the frame after the mouse moves. One pixel: a pointer somewhere else is a pointer
        // somebody moved.
        let (x, y) = game.sim().mouse().at();
        game.sim().mouse().moves_to(x + 1, y);
        let since = game.sim().clock().peek();
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the pointer did not come back on the frame the mouse moved",
        );
        assert_eq!(
            game.sim().mouse().count(),
            0,
            "the counter is not back where it was before orb hid the pointer",
        );

        // And the wait runs again from that movement, which is what makes it a wait for the mouse
        // rather than a countdown from the launch.
        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the second wait was short by the {}µs the first one had already spent",
            STILL_FOR_US - still_for(&game, since),
        );
        game.frames_until("the pointer taken off the screen again", FRAMES, || {
            !game.sim().mouse().shown()
        });
    });
}

/// The pointer comes back when the game's window goes behind, and the wait runs again from the frame it
/// comes forward.
///
/// What orb takes off the screen is the pointer over the game. Whether a counter of this process reaches
/// the pointer over another program's window is not something orb has measured, and a pointer over a
/// window that is not in front is in nobody's way — so a window going behind is answered by putting it
/// back and leaving the mouse to whatever is in front instead.
///
/// Which is not the keyboard's reason for asking the same question: keys read with another window in
/// front are keys somebody typed at that window, and no mouse movement is ever the game's to act on at
/// all.
#[test]
fn the_pointer_comes_back_when_the_games_window_goes_behind() {
    in_its_own_process(|| {
        let game = launched("the-mouse-pointer-behind");
        game.sim().mouse().moves_to(700, 400);
        game.frame();
        game.sim().clock().advance_micros(STILL_FOR_US);
        game.frames_until("the pointer taken off the screen", FRAMES, || {
            !game.sim().mouse().shown()
        });

        // Another program's window in front, with the mouse still where it was left.
        game.sim().display().set_foreground(ANOTHER);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "orb kept the pointer off the screen with the game's window behind",
        );

        // And there it stays, however long nothing moves the mouse.
        game.sim().clock().advance_micros(STILL_FOR_US);
        game.frames(FRAMES);
        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen while another program's window was in front",
        );

        // Forward again, and the wait runs from that frame rather than from the last movement of the
        // mouse — which by now is six seconds ago.
        game.sim().display().set_foreground(WINDOW);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the pointer went on the frame the game's window came forward",
        );
        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the wait after the window came forward was short of the three seconds",
        );
        game.frames_until("the pointer taken off the screen again", FRAMES, || {
            !game.sim().mouse().shown()
        });
    });
}

/// How many asks of the game's own stand against orb's one step, which is a hand moving the mouse
/// across the window for a second or so: `WM_SETCURSOR` arrives per movement of the pointer, and a mouse
/// reports a hundred and twenty-five times a second.
const ASKS: u32 = 125;

/// The game asking for the pointer back is answered by orb rather than by the host.
///
/// 紅魔郷 answers `WM_SETCURSOR` with `ShowCursor(TRUE)` on every movement of the pointer over its
/// window, which is what makes the display counter something orb has to own outright: a call let through
/// raises it without changing the screen, and the one step that takes the pointer off then takes it
/// nowhere. So the ask is answered where the import was patched — the pointer stays where orb put it, and
/// the host is not asked at all.
#[test]
fn the_game_asking_for_the_pointer_back_is_answered_by_orb() {
    in_its_own_process(|| {
        let game = launched("the-mouse-pointer-asked-for");
        game.sim().mouse().moves_to(700, 400);
        game.frame();
        game.sim().clock().advance_micros(STILL_FOR_US);
        game.frames_until("the pointer taken off the screen", FRAMES, || {
            !game.sim().mouse().shown()
        });

        let count = game.sim().mouse().count();
        let asks = game.sim().mouse().asks();
        for _ in 0..ASKS {
            game.answers_wm_setcursor();
        }
        assert!(
            !game.sim().mouse().shown(),
            "the game's own ask put the pointer back on the screen",
        );
        assert_eq!(
            game.sim().mouse().count(),
            count,
            "the game's ask moved the host's display counter, which is the step orb hides with",
        );
        assert_eq!(
            game.sim().mouse().asks(),
            asks,
            "the game's ask reached the host",
        );

        // And a movement still puts the pointer back, which is the half that swallowing the asks must
        // not cost: what shows the pointer is orb's own reading of where it is.
        let (x, y) = game.sim().mouse().at();
        game.sim().mouse().moves_to(x + 1, y);
        game.frame();
        assert!(
            game.sim().mouse().shown(),
            "the pointer did not come back on the frame the mouse moved",
        );
    });
}

/// A launch told not to leaves the pointer to the game.
///
/// `hide_mouse: false`, which is somebody who wants the pointer where it has always been. orb then asks
/// the host for nothing at all — and in a real launch the exe's `ShowCursor` entry is left as the loader
/// wrote it, there being no counter to take over where the pointer is not being taken off.
#[test]
fn a_launch_told_not_to_leaves_the_pointer_to_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach("the-mouse-pointer-off", the_run(), |config| {
            config.hide_mouse = false;
        });
        game.creates_its_window();
        game.sim().mouse().moves_to(700, 400);
        game.frame();

        // Twice the wait, and the mouse where it was left for all of it.
        game.sim().clock().advance_micros(STILL_FOR_US * 2);
        game.frames(FRAMES);
        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen in a launch that was told not to",
        );
        assert_eq!(
            game.sim().mouse().asks(),
            0,
            "orb moved the host's display counter in a launch that was told not to",
        );
    });
}

/// A host that will not say where the pointer is has the pointer left alone.
///
/// `GetCursorPos` fails on a desktop that is not the input one — a locked session is that — and a
/// pointer orb cannot follow is one it cannot tell has stopped. Which is the case for leaving it: a
/// pointer taken off the screen by a launch that then loses sight of the mouse is one nothing brings
/// back.
#[test]
fn a_host_that_will_not_say_where_the_pointer_is_has_it_left_alone() {
    in_its_own_process(|| {
        let game = launched("the-mouse-pointer-unknown");
        game.sim().mouse().refuse(true);
        game.frame();
        game.sim().clock().advance_micros(STILL_FOR_US);
        game.frames(FRAMES);

        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen without being able to read where it was",
        );
        assert_eq!(
            game.sim().mouse().asks(),
            0,
            "orb moved the display counter over a pointer it could not follow",
        );
    });
}
