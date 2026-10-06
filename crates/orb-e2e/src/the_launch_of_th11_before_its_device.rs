//! **A launch of th11 orb is attached to before the game has a Direct3D device, which is every real launch.**
//!
//! What `the_launch_of_th10_before_its_device.rs` asks of that game, asked of this one. `DllMain` runs on the
//! thread the launcher's remote `LoadLibraryW` runs on, long before WinMain's `0x446d30` (called at 0x445750) has
//! `IDirect3D9::CreateDevice` write the device to `0x4c3288` — so orb attaches to a game whose device pointer is
//! still null, and with no hook in front of the call that fills it, `Th11::hooks` declaring no `init_device`.
//! What it has to find the device by is that pointer, read again on a frame.
//!
//! **What th08's file asks and this one does not**, each for what th11's exe and orb's code do:
//!
//! - **the frames a launch spends with no device, which orb hands straight back.** th11 spends none: its one
//!   caller of the frames is WinMain's message loop (0x44587e, 0x44589b, 0x4458a7), which starts after `0x446d30`
//!   has made the device, and which asks the device's `TestCooperativeLevel` before every frame (0x445863).
//! - **the device's `Present` redirected into a letterbox.** That redirect is made at th08's device setup,
//!   `Hooks::init_device`, and th11 declares none and presents through `IDirect3DDevice9` (0x4465e9) — see
//!   `the_window_of_th11.rs`.

use crate::fake::th11::Fake;
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::title;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// orb, attached while the game's device pointer is null, builds its overlay through the device the game made
/// after it and draws over the game with it.
#[test]
fn orb_attached_to_th11_before_its_device_draws_through_the_device_the_game_makes_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device(
            Display::agreed(120),
            "th11-before-its-device",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The window first and the device after it, as WinMain makes them (0x445737, 0x445750).
        game.creates_its_window();
        game.makes_its_device();
        game.frames_until("the overlay", 8, || log.said("overlay: ready"));

        // And now orb draws: the question it puts over the game's own main menu.
        game.its_title_menu_is_up();
        game.frames_until(
            "the main menu taking a press",
            FRAMES_FOR_A_SCREEN,
            || unsafe { orb_core::game::Game::menu_takes_a_press(&orb_core::game::th11::Th11) },
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
