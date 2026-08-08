//! **A launch orb is attached to before the game has a Direct3D device, which is every real launch.**
//!
//! `DllMain` runs on the thread the launcher's remote `LoadLibraryW` runs on, and that is long before
//! `GameWindow::Create` reaches its own tail — so orb attaches to a game whose `g_Supervisor.d3dDevice` is
//! still null, and finds the device through its hook over `GameWindow::InitD3dDevice` (0x421420). That
//! function is called once the device exists and again after every `Reset`: it is all `SetRenderState` on a
//! device that is already there, called from `GameWindow::Create`'s tail, from `GameWindow::Present` where a
//! present failed, and from `main`'s own `D3DERR_DEVICENOTRESET` arm. `src/GameWindow.cpp`, `src/main.cpp`.
//!
//! **A laid-out game with its device already in place never reaches that hook**, which is why this file
//! exists: every other scenario here writes the device before orb is attached, so the hook was dead code and
//! the phase a real launch spends without one was a phase nothing had ever run. See
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
//!
//! What orb does without a device is nothing: `render` hands the frame straight back to the game's own, and
//! `draw_overlay` returns before it has spent one of its tries — so the overlay is still there to be built
//! when the device turns up. Both halves are here, because the first says nothing on its own: a launch that
//! drew nothing and went on drawing nothing would pass it.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::title;
use orb_sim::keys;

/// How many frames the game runs before its Direct3D setup does, which is a phase a real launch spends
/// creating a window and reading its archives.
const BEFORE_THE_DEVICE: u32 = 60;

/// orb draws nothing until the game's device setup runs, and its overlay is ready once it has.
#[test]
fn orb_draws_nothing_until_the_games_device_setup_runs() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device("before-its-device", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // Frames of a game with no device, which orb hands straight back: the game's own front end is built
        // and updated on them, so these are frames orb would have drawn on had it anything to draw through.
        game.forget();
        game.frames(BEFORE_THE_DEVICE);
        assert!(
            !game.log().said("overlay: ready"),
            "orb built an overlay through a device the game has not made yet:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.drawn().quads.is_empty(),
            "orb drew {} quad(s) through a device the game has not made yet",
            game.drawn().quads.len(),
        );

        // `GameWindow::InitD3dDevice`, which is where a real launch's orb finds the device.
        game.finds_its_device();
        game.frames_until("the overlay", 8, || game.log().said("overlay: ready"));
        // And the hook really was the way in: it redirects the device's `Present` into a letterbox, which is
        // the one thing that call does that says out loud it happened.
        assert!(
            game.log()
                .lines()
                .iter()
                .any(|line| line.contains("presenting through a letterbox")),
            "orb did not get in front of the device it was handed:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And now orb draws: the question it puts over the game's own title menu, which is what every other
        // scenario in this tree reads off the screen from the first frame.
        //
        // A frame between the overlay and the press, and it is not politeness. Whether a decide is held back
        // for a question is decided in the frame hook and read by the input hook on the frame *after*, and
        // orb deliberately holds none back while it has no overlay to draw the question with — "a press held
        // back for a question nobody can see is a screen that has stopped working". So the frame the overlay
        // was built on is a frame whose press still goes to the game unanswered.
        game.at_the_title_menu();
        game.frames(2);
        game.press(keys::Z);
        game.one_frame();
        assert!(
            !game.says(title(Menu::Run)).is_empty(),
            "orb drew nothing once the device was there:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
