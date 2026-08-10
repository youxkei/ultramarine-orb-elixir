//! **The rectangle the game's frames really reach the device in.**
//!
//! `the_window.rs` asserts the rectangle orb works out — `window::letterbox()`, the largest 4:3 area that
//! fits the client, centred. What no reading of that can say is whether it is the rectangle Direct3D was
//! given, and that is what this is: the `Present` slot of the device's vtable is where orb's replacement
//! sits, the game's own `GameWindow::Present` calls through it, and what comes out the far side is the ask
//! the driver would have acted on.
//!
//! Three things about that ask:
//!
//! - the **destination** is the letterbox, which is what keeps the game's aspect ratio in a client of any
//!   shape;
//! - the **source** is dropped. The whole back buffer goes into that rectangle, whatever the caller asked
//!   for — a game presenting a part of its surface into a letterbox worked out for the whole of it would be
//!   drawing the wrong part scaled wrongly;
//! - and a **driver that refuses to stretch** is fallen back from. Some do, and what a run must not become
//!   is a window with nothing presented into it: the game's own call goes through instead, which leaves the
//!   run playable and stretched, and orb says so once.

use crate::fake::th06::{ASKS_TO_PRESENT, Fake, Presented, the_run};
use crate::fake::{Launched, Panel, in_its_own_process};
use orb_config::{LogLevel, Screen};
use orb_core::window::letterbox;

/// The size this launch is configured for, on a panel wider than the game's own ratio: what a 4:3 game gets
/// in a 16:9 client is bars down the sides, so the letterbox is a rectangle that is plainly not the client.
const CONFIGURED: Screen = Screen::Window {
    width: 1280,
    height: 720,
};

/// A launch with its window laid out and its device found, which is the one a letterbox is presented in.
///
/// The device turns up afterwards because that is when orb redirects the slot: `init_d3d_device` is the hook
/// it does it from, and a game that had its device before orb attached never reaches it.
fn presenting(name: &str) -> Box<Fake> {
    let game =
        Fake::attach_to_a_panel_before_its_device(Panel::scaled(), name, the_run(), |config| {
            config.screen = CONFIGURED;
            config.log_level = LogLevel::Verbose;
        });
    game.creates_its_window();
    game.finds_its_device();
    assert!(
        game.log()
            .said("screen: presenting through a letterbox, client 1280x720"),
        "the device's Present was not redirected:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// The one present a frame makes.
///
/// # Panics
/// Where the frame made none, or more than one — a frame presented twice is a frame on the screen twice.
fn one_present(game: &Fake) -> Presented {
    let presents = game.presented();
    assert_eq!(
        presents.len(),
        1,
        "the frame asked the device for {} present(s)",
        presents.len(),
    );
    presents[0]
}

/// The whole back buffer into the letterbox, and nothing of what the game asked for.
#[test]
fn a_frame_reaches_the_device_as_the_whole_back_buffer_into_the_letterbox() {
    in_its_own_process(|| {
        let game = presenting("the-letterbox");
        let rectangle = unsafe { letterbox() }.expect("a window laid out on a monitor");
        // Said out loud, because a letterbox that were the whole client would make the assertion below pass
        // for the wrong reason: this game is 4:3 in a 16:9 client, which is bars down the sides.
        assert_eq!(
            (rectangle.width(), rectangle.height()),
            (960, 720),
            "the letterbox is not the 4:3 rectangle a 1280x720 client leaves room for",
        );

        game.forget_presents();
        game.frame();
        assert_eq!(
            one_present(&game),
            Presented {
                source: None,
                destination: Some(rectangle),
            },
            "the frame did not reach the device as the whole back buffer into the letterbox",
        );

        // And the frame after it goes to the same rectangle, the client not having changed: the rectangle is
        // worked out once and kept, which is what a present per frame must not pay for.
        game.forget_presents();
        game.frame();
        assert_eq!(one_present(&game).destination, Some(rectangle));
    });
}

/// And a driver that will not stretch: the game's own call again, said once, and the run still presented.
#[test]
fn a_driver_that_refuses_to_stretch_is_fallen_back_from_and_said_once() {
    in_its_own_process(|| {
        let game = presenting("the-letterbox-refused");
        let log = game.log();
        let rectangle = unsafe { letterbox() }.expect("a window laid out on a monitor");
        game.refuses_to_stretch_on_a_present();

        game.forget_presents();
        game.frame();
        // Two presents for the one frame: the letterbox, which the driver refused, and the game's own ask
        // again — its whole back buffer over the whole client, which is what every launch before orb had.
        assert_eq!(
            game.presented(),
            vec![
                Presented {
                    source: None,
                    destination: Some(rectangle),
                },
                Presented {
                    source: Some(ASKS_TO_PRESENT),
                    destination: None,
                },
            ],
            "the refusal was not fallen back from with the ask the game made",
        );
        assert!(
            log.said("screen: Present into a letterbox failed"),
            "orb did not say the driver refused:\n  {}",
            log.lines().join("\n  ")
        );

        // Said once and not once a frame: a run at sixty frames a second would otherwise write a line for
        // every one of them, which is a log nobody can read the rest of.
        let said = |log: &orb_sim::Log| {
            log.lines()
                .iter()
                .filter(|line| line.contains("Present into a letterbox failed"))
                .count()
        };
        assert_eq!(said(log), 1);
        game.frames(30);
        assert_eq!(
            said(log),
            1,
            "orb said the driver refused once a frame rather than once",
        );

        // And every one of those frames still reached the device, which is the whole point of falling back.
        assert!(
            game.presented().len() > 30,
            "frames stopped being presented once the driver refused",
        );
    });
}
