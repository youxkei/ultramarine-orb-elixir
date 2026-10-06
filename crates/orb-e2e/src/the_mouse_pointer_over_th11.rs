//! **The mouse pointer over th11: gone once the mouse has been still, back the moment it moves** — what
//! `the_mouse_pointer_over_th07` asks of that game, asked of this one.
//!
//! The game is not played with the mouse and orb puts it in a window, which is the launch Windows draws a pointer
//! in. So the pointer sits over the playfield for as long as nobody moves it, and taking it off the screen is
//! orb's to do. **The display counter is one number the whole process shares**, which is why the counter is what
//! these read back rather than a flag — see th07's file.
//!
//! **th07's fourth case, the game asking for the pointer back, is not asked here**: th11's window procedure does
//! answer `WM_SETCURSOR` (0x445e00, `th11/asm.md`'s *The window procedure*), showing the cursor where windowed or
//! inactive, but which call it makes for that has not been read off its exe, and a laid-out game answering it
//! would be a guess written down as the game.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_sim::Clock;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Another program's window in front: any handle that is not the game's.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How long the mouse is left alone before orb takes the pointer off the screen, in microseconds — written out
/// for th07's file's reason: what an e2e test holds is the wait somebody sitting at the game gets.
const STILL_FOR_US: i64 = 3_000_000;

/// How far short of that wait a frame is run, to hold the half of the claim that says the pointer is still there.
const SHORT_OF_IT_US: i64 = 100_000;

/// How many frames the pointer is waited for after that, which is [`SHORT_OF_IT_US`] of them and room over.
const FRAMES: u32 = 16;

/// How long the mouse has been still, as the host's own clock has it.
fn still_for(game: &Fake, since: i64) -> i64 {
    Clock::micros_for_ticks(game.sim().clock().peek() - since)
}

/// The pointer goes once the mouse has been still for three seconds, comes back on the frame after it moves, and
/// goes again three seconds after *that* — the wait being from the last movement rather than from the launch.
#[test]
fn the_pointer_over_th11_goes_once_the_mouse_has_been_still_and_comes_back_when_it_moves() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-mouse-pointer",
            Work::wandering(WORK_US),
        );
        game.one_frame();
        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen on the launch's own first frame",
        );

        game.sim().mouse().moves_to(700, 400);
        let since = game.sim().clock().peek();
        game.one_frame();

        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.one_frame();
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
        let a_frame = game.refresh_period_us() + WORK_US;
        assert!(
            waited < STILL_FOR_US + 2 * a_frame,
            "the pointer went {}µs late, which is more than a frame past the wait",
            waited - STILL_FOR_US,
        );
        assert_eq!(
            game.sim().mouse().count(),
            -1,
            "orb drove the display counter further than the one step that hides the pointer",
        );

        let asked = game.sim().mouse().asks();
        game.frames(FRAMES);
        assert_eq!(
            game.sim().mouse().asks(),
            asked,
            "orb asked the host again with the pointer already off the screen",
        );

        let (x, y) = game.sim().mouse().at();
        game.sim().mouse().moves_to(x + 1, y);
        let since = game.sim().clock().peek();
        game.one_frame();
        assert!(
            game.sim().mouse().shown(),
            "the pointer did not come back on the frame the mouse moved",
        );
        assert_eq!(
            game.sim().mouse().count(),
            0,
            "the counter is not back where it was before orb hid the pointer",
        );

        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.one_frame();
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

/// The pointer comes back when the game's window goes behind, and the wait runs again from the frame it comes
/// forward.
#[test]
fn the_pointer_comes_back_when_th11s_window_goes_behind() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-mouse-pointer-behind",
            Work::wandering(WORK_US),
        );
        game.sim().mouse().moves_to(700, 400);
        game.one_frame();
        game.sim().clock().advance_micros(STILL_FOR_US);
        game.frames_until("the pointer taken off the screen", FRAMES, || {
            !game.sim().mouse().shown()
        });

        game.sim().display().set_foreground(ANOTHER);
        game.frames_until(
            "the pointer back over another program's window",
            FRAMES,
            || game.sim().mouse().shown(),
        );
        assert_eq!(
            game.sim().mouse().count(),
            0,
            "the counter is not back where it was before orb hid the pointer",
        );

        game.sim().clock().advance_micros(STILL_FOR_US * 2);
        game.frames(FRAMES);
        assert!(
            game.sim().mouse().shown(),
            "orb took the pointer off the screen over a window that is not the game's",
        );

        game.sim().display().set_foreground(WINDOW);
        let since = game.sim().clock().peek();
        game.one_frame();
        game.sim()
            .clock()
            .advance_micros(STILL_FOR_US - SHORT_OF_IT_US);
        game.one_frame();
        assert!(
            game.sim().mouse().shown(),
            "the wait after the window came forward was short by the {}µs spent behind",
            STILL_FOR_US - still_for(&game, since),
        );
        game.frames_until("the pointer taken off the screen again", FRAMES, || {
            !game.sim().mouse().shown()
        });
    });
}

/// A launch told not to leaves the pointer to the game.
#[test]
fn a_launch_of_th11_told_not_to_leaves_the_pointer_to_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-mouse-pointer-off",
            Work::wandering(WORK_US),
            |config| config.hide_mouse = false,
        );
        game.sim().mouse().moves_to(700, 400);
        game.one_frame();

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

/// A host that will not say where the pointer is has the pointer left alone — `GetCursorPos` failing on a
/// desktop that is not the input one, which a locked session is.
#[test]
fn a_host_that_will_not_say_where_the_pointer_is_has_it_left_alone_over_th11() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-mouse-pointer-unknown",
            Work::wandering(WORK_US),
        );
        game.sim().mouse().refuse(true);
        game.one_frame();
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
