//! **The d-pad moving the player**, which the game itself cannot do.
//!
//! `Controller::GetControllerInput` reads a pad's two axes and nothing else. Neither of the two fields a
//! d-pad reports in is touched anywhere in it: not winmm's `dwPOV`, at +0x28 of the `JOYINFOEX` it fills
//! on its own stack, and not DirectInput's `rgdwPOV[0]`, at +0x20 of the `DIJOYSTATE2` its other branch
//! fills — the whole function was read for both and reads neither. So a d-pad does nothing at all in the
//! game, while driving the launcher's settings dialog and orb's own menus, both of which read one.
//!
//! `dpad_moves` is what closes that: orb adds the direction the hat is pushed to the word the game's own
//! input read handed back, in the bits that word names the four directions by — 0x10, 0x20, 0x40 and
//! 0x80, read out of the same exe.
//!
//! **What moves the player here is orb's addition and only orb's.** This game does no more with a pad
//! than the real one's arithmetic is left to do — its `get_controller_input` hands the keyboard's word
//! straight back, `docs/adr/0008` refusing to have where-an-axis-becomes-a-direction written twice — so a
//! stick moves nothing in an e2e test and a d-pad that moves the player moved it through orb.
//!
//! Both of the devices the game reads a pad on, because the addition is only as good as the device it was
//! read from: the controller the game polls itself, and the pad winmm has where the game's own
//! enumeration found none.
//!
//! レガシーモード throughout: what a direction does to the player is the same in either mode, and a
//! pointdevice run would put a chapter's frames between the push and the player.

use crate::fake::th06::{Fake, MAPPING, SPEED, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{HAT_AT_REST, Pushed};

/// Where a hat points, in the hundredths of a degree clockwise from straight up that both devices report
/// one in.
const LEFT: u32 = 27000;
const RIGHT: u32 = 9000;

/// How many frames each push is held for. More than one, so that what is asserted is the player moving
/// every frame the direction is held rather than once on its edge.
const HELD: u32 = 3;

/// The pad this plugs into winmm, as an Xbox One pad answers `joyGetDevCapsA` on this machine, and the
/// line that says orb's own thread has read it — the sample being taken off the game's frame, what says a
/// push has arrived is not a number of frames.
const BUTTONS: u32 = 16;
const TRAVEL: (u32, u32) = (0, 65535);
const NAMED: &str = "joystick: winmm 0 is mid=045e pid=02ff";

/// How long an e2e test gives that thread, in frames with a millisecond of real time each.
const WAITS: u32 = 3000;

fn playing(name: &str, settings: impl FnOnce(&mut orb_config::Config)) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        settings(config);
    });
    game.in_a_legacy_run();
    game
}

/// Where the player is, which is what a direction in the word moves.
fn player(game: &Fake) -> (f32, f32) {
    game.image().reproducing_now().player
}

/// The d-pad on the controller the game polls itself moves the player, in the direction it points and by
/// the step a held direction moves them every frame.
#[test]
fn the_dpad_on_the_controller_the_game_polls_moves_the_player() {
    in_its_own_process(|| {
        let game = playing("dpad-on-the-controller", |_| {});

        // Nothing pushed first, the hat where a device that has one leaves it: a player that drifts with
        // the pad at rest is an added direction nobody asked for, and every assertion below would pass.
        let at = player(&game);
        game.push(Pushed::none());
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the player moved with the pad pushed nowhere",
        );

        let at = player(&game);
        game.push(Pushed {
            hat: LEFT,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0 - SPEED * HELD as f32, at.1),
            "the d-pad did not move the player left every frame it was held",
        );

        // And back, which is the assertion that what reached the game was the direction pushed rather
        // than a direction: the hat is one field, and up-for-left is a mapping that works until it does
        // not.
        let at = player(&game);
        game.push(Pushed {
            hat: RIGHT,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0 + SPEED * HELD as f32, at.1),
            "the d-pad did not move the player right",
        );

        // Let go, and the player stands still: what is added is what the hat says now, not what it last
        // said.
        let at = player(&game);
        game.push(Pushed {
            hat: HAT_AT_REST,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the player went on moving after the d-pad was let go",
        );
    });
}

/// And the d-pad of a pad winmm has, where the game's own enumeration found no controller: the other
/// device entirely, sampled on a thread of orb's own.
#[test]
fn the_dpad_of_a_pad_winmm_has_moves_the_player() {
    in_its_own_process(|| {
        let game = Fake::attach("dpad-on-a-winmm-pad", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.sim().joystick().attach(BUTTONS, TRAVEL.0, TRAVEL.1);
        game.in_a_legacy_run();
        waits_until(&game, "orb's own thread reading the pad", || {
            game.log().said(NAMED)
        });

        // Pushed left, and waited for rather than counted in frames: the sample is taken every four
        // milliseconds by a thread nothing here drives, so what says the push has arrived is the player
        // having moved on it.
        let at = player(&game);
        game.sim().joystick().pushes_the_hat(LEFT);
        waits_until(&game, "the d-pad moving the player", || {
            player(&game).0 < at.0
        });
        assert_eq!(
            player(&game).1,
            at.1,
            "a hat pushed left moved the player down the screen as well",
        );

        // And let go, which says the direction came out of the sample being taken now rather than out of
        // one taken once: the player stops on a frame of their own accord.
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        let mut stopped = false;
        for _ in 0..WAITS {
            let was = player(&game);
            game.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
            if player(&game) == was {
                stopped = true;
                break;
            }
        }
        assert!(
            stopped,
            "the player went on moving after the d-pad was let go:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// `dpad_moves: false` hands the game the word its own read produced, which is a d-pad that does nothing —
/// the game as it is without orb.
#[test]
fn the_setting_off_leaves_the_pad_as_the_game_has_it() {
    in_its_own_process(|| {
        let game = playing("dpad-off", |config| config.dpad_moves = false);

        let at = player(&game);
        game.push(Pushed {
            hat: LEFT,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the d-pad moved the player in a launch that turned it off",
        );

        // And the pad still answers for what it always answered for: the buttons the game's own mapping
        // names, which this setting has nothing to do with. The shot button held is a run being shot
        // through, and the player standing still under it is the whole of what was turned off.
        game.push(Pushed::button(MAPPING.shoot));
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the player moved on a button that is not a direction",
        );
    });
}

/// Frames of the game with a millisecond of real time each, until `done`.
///
/// What an e2e test does about the sampling thread: a push on winmm's device reaches the game through a
/// thread nothing here drives, so it is waited for rather than counted.
///
/// # Panics
/// After [`WAITS`], with the log.
fn waits_until(game: &Fake, what: &str, done: impl Fn() -> bool) {
    for _ in 0..WAITS {
        if done() {
            return;
        }
        game.frame();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!(
        "{what} did not happen in {WAITS} frame(s):\n  {}",
        game.log().lines().join("\n  ")
    );
}
