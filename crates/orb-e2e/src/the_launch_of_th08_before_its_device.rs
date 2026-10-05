//! **A launch of th08 orb is attached to before the game has a Direct3D device, which is every real launch.**
//!
//! What `the_launch_of_th07_before_its_device.rs` asks of that game, asked of this one. `DllMain` runs on the
//! thread the launcher's remote `LoadLibraryW` runs on, long before the game's own startup has made a device —
//! so orb attaches to a game whose device pointer is still null, and finds the device through its hook over
//! the device setup at 0x4424c0, which is also where the device's `Present` is redirected into a letterbox.
//! That setup is what makes the device (`IDirect3D8::CreateDevice` into `[0x17ce760]`), so the device is
//! there to be found only once the call it stands in front of has returned.
//!
//! **`th08.rs`'s letterbox tests attach before the device too, and find it before a single frame**: so the
//! phase a real launch spends without one — frames orb hands straight back, and an overlay not yet built — is a
//! phase nothing there runs. Both halves are here, because the first says nothing on its own: a launch that drew
//! nothing and went on drawing nothing would pass it.

use crate::fake::th08::Fake;
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::title;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the game runs before its Direct3D setup does, which is a phase a real launch spends
/// creating a window and unpacking `th08.dat`.
const BEFORE_THE_DEVICE: u32 = 60;

/// How many frames a screen or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 120;

/// orb draws nothing until the game's device setup runs, and its overlay is ready once it has.
#[test]
fn orb_draws_nothing_over_th08_until_the_games_device_setup_runs() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device(
            Display::agreed(120),
            "th08-before-its-device",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The window first, as a launch has it: the letterbox is worked out from the client area of a window
        // that exists, and the device comes after it.
        game.creates_its_window();

        // Frames of a game with no device, which orb hands straight back.
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
        assert!(
            log.said("presenting through a letterbox"),
            "orb did not get in front of the device it was handed:\n  {}",
            log.lines().join("\n  ")
        );

        // And now orb draws: the question it puts over the game's own title menu. The title placed, and
        // left to take a press — this game holds one back over a screen that has just come up (0x467cac).
        game.its_title_menu_is_up();
        game.frames_until(
            "the title menu taking a press",
            FRAMES_FOR_A_SCREEN,
            || unsafe { orb_core::game::Game::menu_takes_a_press(&orb_core::game::th08::Th08) },
        );
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
