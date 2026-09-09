//! **A launch of th07 orb is attached to before the game has a Direct3D device, which is every real launch.**
//!
//! `DllMain` runs on the thread the launcher's remote `LoadLibraryW` runs on, long before the game's own
//! startup has made a device — so orb attaches to a game whose device pointer at 0x575958 is still null, and
//! finds the device through its hook over `GameWindow::InitD3dDevice` at 0x4356a0, which is all render
//! states on a device that already exists. That hook is also where the device's `Present` is redirected into
//! a letterbox, which is the one thing the call does that says out loud it happened.
//!
//! **A laid-out game with its device already in place never reaches that hook**, which is why this file
//! exists: every other e2e test over this game writes the device before orb is attached, so the phase a real
//! launch spends without one is a phase nothing had run. See
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
//!
//! What orb does without a device is nothing: the frame is handed straight back to the game's own, and the
//! overlay is not built until there is a device to build it through — so it is still there to be built when
//! the device turns up. Both halves are here, because the first says nothing on its own: a launch that drew
//! nothing and went on drawing nothing would pass it.

use crate::fake::th07::Fake;
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::title;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the game runs before its Direct3D setup does, which is a phase a real launch spends
/// creating a window and reading its archives.
const BEFORE_THE_DEVICE: u32 = 60;

/// orb draws nothing until the game's device setup runs, and its overlay is ready once it has.
#[test]
fn orb_draws_nothing_over_th07_until_the_games_device_setup_runs() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device(
            Display::agreed(120),
            "th07-before-its-device",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The window first, as a launch has it: the letterbox is worked out from the client area of a window
        // that exists, and the device comes after it.
        game.creates_its_window();

        // Frames of a game with no device, which orb hands straight back: these are frames orb would have
        // drawn on had it anything to draw through.
        game.forget();
        game.frames(BEFORE_THE_DEVICE);
        assert!(
            !log.said("overlay: ready"),
            "orb built an overlay through a device the game has not made yet:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.drawn().quads.is_empty(),
            "orb drew {} quad(s) through a device the game has not made yet",
            game.drawn().quads.len(),
        );

        // `GameWindow::InitD3dDevice`, which is where a real launch's orb finds the device.
        game.finds_its_device();
        game.frames_until("the overlay", 8, || log.said("overlay: ready"));
        // And the hook really was the way in: it redirects the device's `Present` into a letterbox.
        assert!(
            log.said("presenting through a letterbox"),
            "orb did not get in front of the device it was handed:\n  {}",
            log.lines().join("\n  ")
        );

        // And now orb draws: the question it puts over the game's own title menu, which is what every other
        // e2e test over this game reads off the screen from the first frame.
        //
        // Frames between the overlay and the press, and it is not politeness: whether a decide is held back
        // for a question is decided in the frame hook and read by the input hook on the frame *after*, and
        // orb holds none back while it has no overlay to draw the question with.
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        game.frames(2);
        game.press_until(keys::Z, "the mode question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        assert!(
            !game
                .a_frame_that_says(title(Menu::Run, LANGUAGE))
                .is_empty(),
            "orb drew nothing once the device was there:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
