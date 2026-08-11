//! **The pad half of the game's own input read, which is orb's.**
//!
//! `Controller::GetControllerInput` (0x41cfc0) is a tail call inside `Controller::GetInput` — from its two
//! exits, 0x41dc78 and 0x41e09d, and from nowhere else in the exe — and orb hooks it and does not call
//! through. Everything a pad does to the word the game acts on is orb's answer: the device the game's own
//! enumeration found as much as the pads it has none for.
//!
//! **Why it is not the game's any more.** Adding to the word the game read leaves behind the one thing
//! that cannot be added: state the game keeps and feeds from the device it holds.
//! `g_IsEigthFrameOfHeldInput` (0x69d8f4) is that state — where the mapping puts focus and shoot on one
//! button the game holds the player still off the count of frames that button has been held, and a pad the
//! game has no device for never reaches the count. With the read itself here, there is one count and one
//! place that decides.
//!
//! **What is asserted here is the arithmetic itself**, which is what makes this file the other half of
//! `a_pad_the_game_has_no_device_for.rs`: that one is about which pads are read, and this one about what
//! reading one produces. Both of the game's own rules are in it — a device it holds is measured against
//! `cfg.padXAxis` and `cfg.padYAxis` in the ±1000 it gave those axes, and every other pad against the
//! travel its own caps report — and both are read off the same function.
//!
//! **This game does no pad arithmetic of its own**, which is what leaves the assertions here about orb:
//! its `get_controller_input` handed the keyboard's word straight back for as long as the arithmetic was
//! the real game's, and `docs/adr/0008` is why it never had a copy of it. So a button that reaches the
//! word reached it through orb.
//!
//! レガシーモード throughout: what a direction does to the player is the same in either mode, and a
//! pointdevice run would put a chapter's frames between the push and the player.

use crate::fake::th06::{Fake, MAPPING, SPEED, button, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{HAT_AT_REST, Mapping, Pushed};

/// How many frames each push is held for, where what is being watched is the player: more than one, so
/// that what is asserted is a direction held rather than an edge.
const HELD: u32 = 3;

/// Where the player is, which is what a direction in the word moves.
fn player(game: &Fake) -> (f32, f32) {
    game.image().reproducing_now().player
}

/// A legacy run with the game holding the controller its own enumeration found, and nothing else plugged
/// in anywhere.
fn playing(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.in_a_legacy_run();
    game
}

/// The buttons of the device the game holds reach the word, each as the bit the game's own mapping names
/// it by.
#[test]
fn the_buttons_of_the_pad_the_game_holds_reach_the_word() {
    in_its_own_process(|| {
        let game = playing("the-pad-the-game-holds-buttons");

        for (button, bit) in [
            (MAPPING.shoot, button::SHOOT),
            (MAPPING.bomb, button::BOMB),
            (MAPPING.focus, button::FOCUS),
            (MAPPING.menu, button::MENU),
            (MAPPING.up, button::UP),
            (MAPPING.down, button::DOWN),
        ] {
            game.push(Pushed::button(button));
            game.frame();
            assert_eq!(
                game.image().input_now(),
                bit,
                "button {button} did not reach the word as {bit:#x} alone",
            );
        }

        // And a button the mapping names nothing for reaches it as nothing: this game maps six of the
        // nine, and leaves the directions and skip unmapped the way the game's own defaults do.
        game.push(Pushed::button(9));
        game.frame();
        assert_eq!(
            game.image().input_now(),
            0,
            "a button the game's mapping names nothing for reached the word",
        );
    });
}

/// And its stick moves the player, each axis past the threshold the configuration keeps for that axis —
/// which is the rule for a device the game holds, and not the one every other pad is measured by.
#[test]
fn the_stick_of_the_pad_the_game_holds_moves_the_player() {
    in_its_own_process(|| {
        let game = playing("the-pad-the-game-holds-stick");

        // Inside both thresholds is a stick nobody is pushing.
        let at = player(&game);
        game.push(Pushed {
            x: i32::from(MAPPING.x_axis),
            y: i32::from(MAPPING.y_axis),
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "a stick inside its own dead zone moved the player",
        );

        // Past X's, which is the axis across the screen.
        game.push(Pushed {
            x: -i32::from(MAPPING.x_axis) - 1,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0 - SPEED * HELD as f32, at.1),
            "the stick pushed left did not move the player left every frame",
        );

        // And past Y's, which is measured down the screen. Pushed to X's threshold at the same time,
        // which is inside Y's: an axis read against the other's threshold would move the player twice.
        let at = player(&game);
        game.push(Pushed {
            x: i32::from(MAPPING.x_axis),
            y: i32::from(MAPPING.y_axis) + 1,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0, at.1 + SPEED * HELD as f32),
            "the stick pushed down did not move the player down alone",
        );
    });
}

/// One button that is both shoot and focus holds the player still after the frames the game counts, on the
/// device the game holds as much as on a pad it has none for.
///
/// The count is `g_IsEigthFrameOfHeldInput` at 0x69d8f4: raised while the button is held and read against
/// eight, which is what makes holding the shot button a way to move slowly.
#[test]
fn holding_one_button_that_is_both_holds_the_player_still() {
    in_its_own_process(|| {
        let game = playing("the-pad-the-game-holds-held-shot");
        game.image().maps_the_pad(Mapping {
            focus: MAPPING.shoot,
            ..MAPPING
        });

        // The first frame of the hold shoots and does not hold the player still.
        game.push(Pushed::button(MAPPING.shoot));
        game.frame();
        assert_eq!(
            game.image().input_now(),
            button::SHOOT,
            "the player was held still on the first frame the button was held",
        );

        // Nor does the frame before the one the game counts to.
        game.frames(FOCUSES_AFTER - 2);
        assert_eq!(
            game.image().input_now(),
            button::SHOOT,
            "the player was held still before the frame the game holds them still on",
        );
        game.frame();
        assert_eq!(
            game.image().input_now(),
            button::SHOOT | button::FOCUS,
            "holding the one button that is both did not hold the player still",
        );

        // And the count is the game's own field, so a look at that field says the same thing: eight of
        // them, which is where the player is held still from.
        assert_eq!(
            game.image().held_shot_frames(),
            FOCUSES_AFTER as u16,
            "the count of held frames is not in the field the game keeps it in",
        );

        // Let go long enough and it is neither again.
        game.push(Pushed::none());
        game.frames(FOCUSES_AFTER);
        assert_eq!(
            game.image().input_now(),
            0,
            "the player was held still after the button was let go",
        );
    });
}

/// Which frame of holding the player is held still from: `g_IsEigthFrameOfHeldInput` counted up to 16 and
/// read against 8, at 0x41d06e and 0x41d08e in the branch that reads winmm and 0x41d3a4 and 0x41d3c3 in
/// the one that polls the device the game holds.
const FOCUSES_AFTER: u32 = 8;

/// And what that read costs reaches the `perf:` line, which is what the span orb's hook records is for.
///
/// The read is a tail call inside `Controller::GetInput` and cannot be told apart from outside it, so
/// timing it means standing in front of it — which orb does anyway now, the pad half being its own. What
/// this file's head records about the old read is what that line said, and a launch whose perf line has no
/// joystick span in it is a launch where nobody could have found it out.
///
/// Verbose because that is what writes the line, and not because it is what installs anything: the hook
/// goes in on every launch, a launch without it being one where no pad does anything at all.
#[test]
fn the_span_that_read_costs_reaches_the_perf_line() {
    in_its_own_process(|| {
        let game = playing("the-pad-half-the-perf-line");
        game.frames_until_the_log_holds_another("perf:");
        let perf = game
            .log()
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("perf:"))
            .expect("the perf line just waited for");
        assert!(
            perf.contains("joystick="),
            "the perf line has no joystick span in it: {perf}",
        );
        // Beside the read it is *inside* and not additional to it, which is the one thing the line has to
        // say for the number to be readable at all.
        assert!(
            perf.contains("input="),
            "the perf line has a joystick span and no input span to be inside: {perf}",
        );
    });
}

/// A device whose poll fails is a device nothing is read off, and the word is what the keyboard read made
/// it — with the acquire that gets it back asked for and the frame given up.
///
/// The game's own read loops on that acquire up to 400 times (0x41d2a5 to 0x41d2f0). orb asks once and
/// gives the frame up, which is [`Th06::controller_state`]'s decision and not a divergence in what a
/// working device answers: a menu of orb's already did it that way, and the next frame is a sixtieth of a
/// second off.
#[test]
fn a_controller_whose_poll_fails_leaves_the_word_as_the_keyboard_read_it() {
    in_its_own_process(|| {
        let game = playing("the-pad-the-game-holds-lost");

        game.push(Pushed::button(MAPPING.shoot));
        game.frame();
        assert_eq!(game.image().input_now(), button::SHOOT);

        let acquires = game.controller_acquires();
        game.its_controller_poll_fails(true);
        game.frame();
        assert_eq!(
            game.image().input_now(),
            0,
            "a device whose poll failed still put buttons in the word",
        );
        assert_eq!(
            game.controller_acquires(),
            acquires + 1,
            "the lost device was not asked for again",
        );

        // And back, once the poll answers again: the same button is in the word without anything having
        // been pressed again.
        game.its_controller_poll_fails(false);
        game.frame();
        assert_eq!(
            game.image().input_now(),
            button::SHOOT,
            "the device that answered again was not read",
        );
    });
}

/// The hat is the one thing on that device the game reads on no device at all, which is why it is behind
/// `dpad_moves` where everything above is not: what the rest of this file asserts is what the game would
/// have produced from the same pad, and a d-pad is what it would not.
#[test]
fn the_setting_off_leaves_only_the_hat_out_of_the_word() {
    in_its_own_process(|| {
        let game = Fake::attach("the-pad-the-game-holds-dpad-off", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.dpad_moves = false;
        });
        game.in_a_legacy_run();

        let at = player(&game);
        game.push(Pushed {
            hat: 27000,
            ..Pushed::none()
        });
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the d-pad moved the player in a launch that turned it off",
        );

        // And everything else of that pad still reaches the word, this setting being about the hat.
        game.push(Pushed {
            hat: HAT_AT_REST,
            ..Pushed::button(MAPPING.shoot)
        });
        game.frame();
        assert_eq!(
            game.image().input_now(),
            button::SHOOT,
            "the shot button was left out of the word with the d-pad turned off",
        );
    });
}
