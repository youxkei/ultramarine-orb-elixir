//! **The pads the game cannot see**, which on this machine is the pad somebody is holding.
//!
//! 紅魔郷 holds exactly one game controller and settles which at startup: `g_Supervisor.controller`
//! (0x6c6d2c) is written in one place in the whole exe — the enumeration's own callback at 0x423da0,
//! which calls `CreateDevice` only while the pointer is still null and returns `DIENUM_STOP` the moment
//! it succeeds. So the first creatable attached controller wins, nothing else is ever made, and the one
//! other device it will read is winmm's joystick 0. Three pads follow that the game can do nothing with:
//! a second pad, a pad plugged in after the game started, and — on the machine orb was measured on — the
//! pad itself, which sits in XInput's second slot while winmm's joystick 0 holds the phantom Windows
//! leaves there.
//!
//! This is a one-player game, so every pad on the machine is that one player's. orb reads all of them
//! and adds **the one last pushed** to the word the game's own input read handed back: its buttons
//! through the game's own mapping, its stick as the game's own read makes a direction of an axis, and
//! its hat where `dpad_moves` asks for it.
//!
//! **The last pushed and not all of them merged**, which is what two pads in front of one person means:
//! the other one is on the floor, and a device sitting there with an axis drifting past its dead zone
//! would otherwise hold a direction down for the whole run. Whichever pad was last done something to is
//! the one in somebody's hands.
//!
//! **What moves the player here is orb's addition and only orb's.** This game does no more with a pad
//! than the real one's arithmetic is left to do — its `get_controller_input` hands the keyboard's word
//! straight back, `docs/adr/0008` refusing to have where-an-axis-becomes-a-direction written twice — so
//! a pad that moved the player moved it through orb.
//!
//! レガシーモード throughout: what a direction does to the player is the same in either mode, and a
//! pointdevice run would put a chapter's frames between the push and the player.

use crate::fake::th06::{Fake, MAPPING, SPEED, button, the_run};
use crate::fake::{LANGUAGE, Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th06::image::Mapping;
use orb_core::mode::{Mode, title};
use orb_sim::{POV_CENTERED, button as xinput};

/// The pad this plugs into winmm, as an Xbox One pad answers `joyGetDevCapsA` on this machine.
const BUTTONS: u32 = 16;
const TRAVEL: (u32, u32) = (0, 65535);

/// Where a hat points, in the hundredths of a degree clockwise from straight up that winmm reports one
/// in — which is how the pads only orb reads are pushed, XInput's own d-pad bits being what
/// `orb_sim::button` holds.
const DOWN: u32 = 18000;

/// The slot the pad on this machine sits in — XInput's second, which is the one that leaves winmm with
/// nothing but the phantom.
const SECOND_SLOT: u32 = 1;
/// And another, for the two-pads-in-front-of-one-person question.
const THIRD_SLOT: u32 = 2;

/// The line that says orb's own thread has read a pad in a slot, which is what an e2e test waits for
/// rather than counting frames: the sample is taken off the game's frame.
fn read_the_slot(slot: u32) -> String {
    format!("joystick: xinput {slot} has a pad")
}

/// How long an e2e test gives that thread, in frames with a millisecond of real time each.
const WAITS: u32 = 3000;

/// Where the player is, which is what a direction in the word moves.
fn player(game: &Fake) -> (f32, f32) {
    game.image().reproducing_now().player
}

/// A legacy run with the game holding a controller of its own — the pad it enumerated at startup, at
/// rest throughout — and a pad in XInput's second slot that the game has no device for.
fn playing_with_a_second_pad(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.sim().xinput().attach(SECOND_SLOT);
    game.in_a_legacy_run();
    waits_until(&game, "orb's own thread reading the pad", || {
        game.log().said(&read_the_slot(SECOND_SLOT))
    });
    game
}

/// The d-pad of a pad the game has no device for moves the player, in the direction it points.
#[test]
fn the_dpad_of_a_pad_the_game_has_no_device_for_moves_the_player() {
    in_its_own_process(|| {
        let game = playing_with_a_second_pad("a-second-pads-dpad");

        // Nothing pushed on either pad first: a player that drifts is an added direction nobody asked
        // for, and every assertion below would pass.
        let at = player(&game);
        for _ in 0..READS_KEYS_AFTER {
            game.frame();
        }
        assert_eq!(
            player(&game),
            at,
            "the player moved with both pads pushed nowhere",
        );

        // Pushed left, and waited for rather than counted in frames: the sample is taken every four
        // milliseconds by a thread nothing here drives, so what says the push has arrived is the player
        // having moved on it.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::DPAD_LEFT);
        waits_until(&game, "the d-pad moving the player left", || {
            player(&game).0 < at.0
        });
        assert_eq!(
            player(&game).1,
            at.1,
            "a hat pushed left moved the player down the screen as well",
        );

        // And every frame it is held, which is what a direction in the word does rather than an edge.
        let at = player(&game);
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0 - SPEED * HELD as f32, at.1),
            "the d-pad did not move the player left every frame it was held",
        );

        // And back, which says what reached the game was the direction pushed rather than a direction.
        let at = player(&game);
        game.sim().xinput().pushes(SECOND_SLOT, xinput::DPAD_RIGHT);
        waits_until(&game, "the d-pad moving the player right", || {
            player(&game).0 > at.0
        });

        // Let go, and the player stands still: what is added is what the pad says now.
        game.sim().xinput().pushes(SECOND_SLOT, 0);
        frames_until(&game, "the player standing still again", || {
            let was = player(&game);
            game.frame();
            player(&game) == was
        });
    });
}

/// How many frames a push is held for where the pad is not being waited on.
const HELD: u32 = 3;

/// And its stick moves the player too, past the dead zone the game's own read puts on an axis.
///
/// The stick and not only the hat because a pad the game has no device for is one orb stands in for the
/// whole read of: the game would have read both axes of it, and `dpad_moves` has nothing to do with
/// either.
#[test]
fn the_stick_of_a_pad_the_game_has_no_device_for_moves_the_player() {
    in_its_own_process(|| {
        let game = playing_with_a_second_pad("a-second-pads-stick");

        let at = player(&game);
        // XInput's Y is measured upwards, which is the opposite of every axis winmm reports.
        game.sim()
            .xinput()
            .pushes_the_stick(SECOND_SLOT, 0, i16::MIN);
        waits_until(&game, "the stick moving the player down", || {
            player(&game).1 > at.1
        });
        assert_eq!(
            player(&game).0,
            at.0,
            "a stick pushed down moved the player across the screen as well",
        );

        // And centred, where nothing is being pushed at all: a stick sitting in the middle is not a
        // direction, and a run played with two directions held is what believing one costs.
        game.sim().xinput().pushes_the_stick(SECOND_SLOT, 0, 0);
        frames_until(&game, "the player standing still again", || {
            let was = player(&game);
            game.frame();
            player(&game) == was
        });
    });
}

/// Its buttons reach the run as well, through the game's own mapping: somebody holding a pad shoots and
/// bombs with it, and a pad that only steered would be half a pad.
#[test]
fn the_buttons_of_a_pad_the_game_has_no_device_for_reach_the_word() {
    in_its_own_process(|| {
        let game = playing_with_a_second_pad("a-second-pads-buttons");

        // XInput reports its buttons in its own order and the game's mapping names them in
        // DirectInput's, so A is the button the mapping calls shoot — whatever the player mapped.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::A);
        frames_until(&game, "the shoot button reaching the word", || {
            game.image().input_now() & button::SHOOT != 0
        });

        game.sim().xinput().pushes(SECOND_SLOT, xinput::B);
        frames_until(&game, "the bomb button reaching the word", || {
            game.image().input_now() & button::BOMB != 0
        });

        // And a button the mapping names nothing for does nothing: this game maps six, and Back is
        // DirectInput's button 6.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::BACK);
        frames_until(&game, "the pad letting the bomb button go", || {
            game.image().input_now() & button::BOMB == 0
        });
        for _ in 0..READS_KEYS_AFTER {
            game.frame();
            assert_eq!(
                game.image().input_now(),
                0,
                "a button the game's mapping names nothing for reached the word",
            );
        }
    });
}

/// Focus is one of those buttons where the mapping gives it one of its own, which is what the game's own
/// defaults do.
#[test]
fn the_focus_button_of_such_a_pad_holds_the_player_still() {
    in_its_own_process(|| {
        let game = playing_with_a_second_pad("a-second-pads-focus-button");

        // The right shoulder, which is DirectInput's button 5 — this game's focus button.
        game.sim()
            .xinput()
            .pushes(SECOND_SLOT, xinput::RIGHT_SHOULDER);
        frames_until(&game, "the focus button reaching the word", || {
            game.image().input_now() & button::FOCUS != 0
        });
        // And on its own, with nothing else in the word: focus is not the shot button here.
        assert_eq!(
            game.image().input_now(),
            button::FOCUS,
            "the focus button reached the word as something else as well",
        );
    });
}

/// And where the mapping puts focus and shoot on **one** button, holding it holds the player still after
/// the frames the game waits — which is what that configuration is for, and what a pad the game cannot see
/// has to do as well as the one it can.
///
/// The game does not read a focus button at all there: it counts the frames the shot button has been held,
/// in `g_IsEigthFrameOfHeldInput` at 0x69d8f4, and holds the player still from the eighth of them. That
/// count is driven by the game's own read of its own device, which a pad orb read for itself never
/// reaches, so orb counts the frames of its own pad and answers with the same numbers.
#[test]
fn holding_one_button_that_is_both_shoots_and_then_holds_the_player_still() {
    in_its_own_process(|| {
        let game = playing_with_a_second_pad("a-second-pads-held-shoot");
        // Focus on the shot button, which is the configuration this is about.
        game.image().maps_the_pad(Mapping {
            focus: MAPPING.shoot,
            ..MAPPING
        });

        // Held, and the first frame it reaches the word is the first frame of the count: shooting, and
        // not yet holding the player still.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::A);
        frames_until(&game, "the shot button reaching the word", || {
            game.image().input_now() & button::SHOOT != 0
        });
        assert_eq!(
            game.image().input_now() & button::FOCUS,
            0,
            "the player was held still on the first frame the button was held",
        );

        // The frames the game waits, less the one already counted. Nothing yet on the last of them.
        game.frames(FOCUSES_AFTER - 2);
        assert_eq!(
            game.image().input_now() & (button::SHOOT | button::FOCUS),
            button::SHOOT,
            "the player was held still before the frame the game holds them still on",
        );
        game.frame();
        assert_eq!(
            game.image().input_now() & (button::SHOOT | button::FOCUS),
            button::SHOOT | button::FOCUS,
            "holding the one button that is both did not hold the player still",
        );

        // Let go for long enough and it is neither again: the count runs back down, which is what makes
        // this a hold rather than a latch.
        game.sim().xinput().pushes(SECOND_SLOT, 0);
        frames_until(&game, "the button letting go", || {
            game.image().input_now() == 0
        });
        for _ in 0..FOCUSES_AFTER {
            game.frame();
            assert_eq!(
                game.image().input_now(),
                0,
                "the player was held still after the button was let go",
            );
        }
    });
}

/// Which frame of holding the game holds the player still from: `g_IsEigthFrameOfHeldInput` counted up
/// to 16 and read against 8, at 0x41d06e and 0x41d08e in the branch that reads winmm and 0x41d3a4 and
/// 0x41d3c3 in the one that polls its controller.
const FOCUSES_AFTER: u32 = 8;

/// **The pad the run is played with is the one last pushed**, which is what two pads in front of one
/// person means: the other is on the floor.
///
/// Merging both would be a run played with whatever the one nobody is holding happens to be doing —
/// and a pad left with an axis drifting past its dead zone holds a direction down for the whole run.
#[test]
fn the_run_is_played_with_the_pad_last_pushed() {
    in_its_own_process(|| {
        let game = Fake::attach("two-pads", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.sim().xinput().attach(SECOND_SLOT);
        game.sim().xinput().attach(THIRD_SLOT);
        game.in_a_legacy_run();
        for slot in [SECOND_SLOT, THIRD_SLOT] {
            waits_until(&game, "orb's own thread reading both pads", || {
                game.log().said(&read_the_slot(slot))
            });
        }

        // One of them pushed left and held there for the rest of this test.
        let at = player(&game);
        game.sim().xinput().pushes(SECOND_SLOT, xinput::DPAD_LEFT);
        waits_until(&game, "the first pad moving the player left", || {
            player(&game).0 < at.0
        });

        // And the other pushed right while the first is still held left: the player goes right, which is
        // the pad that was just picked up winning outright rather than the two cancelling out.
        let at = player(&game);
        game.sim().xinput().pushes(THIRD_SLOT, xinput::DPAD_RIGHT);
        waits_until(&game, "the pad just pushed moving the player right", || {
            player(&game).0 > at.0
        });
        let at = player(&game);
        game.frames(HELD);
        assert_eq!(
            player(&game),
            (at.0 + SPEED * HELD as f32, at.1),
            "the two pads' directions were merged rather than the last pushed being read",
        );

        // The second let go, and the player stands still although the first is still held left: what is
        // read is the pad last pushed, and letting go of it is the last thing that happened.
        game.sim().xinput().pushes(THIRD_SLOT, 0);
        frames_until(&game, "the player standing still", || {
            let was = player(&game);
            game.frame();
            player(&game) == was
        });
        let at = player(&game);
        game.frames(HELD);
        assert_eq!(
            player(&game),
            at,
            "the pad nobody had touched since was read again",
        );

        // And the first pad pushed somewhere else, which is somebody picking it up again.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::DPAD_DOWN);
        waits_until(&game, "the first pad moving the player again", || {
            player(&game).1 > at.1
        });
    });
}

/// A pad winmm has at an index the game never asks about drives the run too: the game asks winmm for
/// joystick 0 alone, and a second pad is at an index of its own.
#[test]
fn a_pad_at_an_index_the_game_never_asks_about_drives_the_run() {
    in_its_own_process(|| {
        let game = Fake::attach("a-pad-winmm-has-at-index-one", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // No controller and nothing on joystick 0, which is everything the game itself can read.
        game.image().no_controller();
        game.sim()
            .joystick()
            .attach_at(ANOTHER_INDEX, BUTTONS, TRAVEL.0, TRAVEL.1);
        game.in_a_legacy_run();
        waits_until(&game, "orb's own thread reading that pad", || {
            game.log().said(&format!(
                "joystick: winmm {ANOTHER_INDEX} is mid=045e pid=02ff"
            ))
        });

        let at = player(&game);
        game.sim().joystick().pushes_the_hat_at(ANOTHER_INDEX, DOWN);
        waits_until(&game, "the pad moving the player down", || {
            player(&game).1 > at.1
        });
        game.sim()
            .joystick()
            .pushes_the_hat_at(ANOTHER_INDEX, POV_CENTERED);
        frames_until(&game, "the player standing still again", || {
            let was = player(&game);
            game.frame();
            player(&game) == was
        });
    });
}

/// The index that pad is at: not 0, which is the one the game asks winmm about.
const ANOTHER_INDEX: u32 = 1;

/// And a question of orb's own is answered on it — on the machine orb was measured on, where the game
/// has no controller, winmm's joystick 0 is the phantom, and the pad is in XInput's second slot.
///
/// The phantom is a device that answers `joyGetPosEx` with every field zero: `mid=413d pid=2104`, no
/// buttons and no axes. So the whole of what the game itself can reach is a device that cannot say
/// anything, and before orb read XInput the pad drove nothing at all.
#[test]
fn a_question_of_orbs_is_answered_on_the_pad_the_game_cannot_reach() {
    in_its_own_process(|| {
        let game = Fake::attach("a-pad-only-xinput-has", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.sim().joystick().attach_a_phantom();
        game.sim().xinput().attach(SECOND_SLOT);
        game.at_the_title_menu();
        waits_until(&game, "orb's own thread reading the pad", || {
            game.log().said(&read_the_slot(SECOND_SLOT))
        });

        // The question up, on a keypress, which is the only way it goes up.
        game.press(orb_sim::keys::Z);
        game.one_frame();
        assert!(
            !game.says(title(Menu::Run, LANGUAGE)).is_empty(),
            "the press did not put the question up:\n  {}",
            game.log().lines().join("\n  ")
        );
        game.frames(READS_KEYS_AFTER);

        // Down to レガシーモード on the pad's own d-pad, and answered on the button the game's mapping
        // calls shoot.
        game.sim().xinput().pushes(SECOND_SLOT, xinput::DPAD_DOWN);
        waits_until(&game, "the pad moving the cursor", || {
            under_the_cursor(&game) == Mode::Normal
        });
        game.sim().xinput().pushes(SECOND_SLOT, xinput::A);
        waits_until(&game, "the pad's answer", || outcomes(&game) == 1);
        assert!(
            game.log().said(&format!(
                "mode: answered on the {}",
                orb_core::menu::By::Pad
            )),
            "the answer was not named as the pad's:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for mode in orb_core::mode::CHOICES {
        let text = mode.name(LANGUAGE);
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == orb_core::menu_ui::SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        }
    }
    on.expect("neither mode is lit")
}

/// How many outcomes orb has reported for the question — counted rather than looked for, a line once
/// written staying written.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// Frames of the game with a millisecond of real time each, until `done`.
///
/// What an e2e test does about the sampling thread: a push on a pad the game has no device for reaches
/// the game through a thread nothing here drives, so it is waited for rather than counted.
///
/// **A frame per turn and none inside `done`**, which matters wherever what is asserted afterwards counts
/// frames: a `done` that ran one of its own would leave the wait ending a frame past the first one its
/// answer was true on, and how far past would be however the thread happened to be scheduled.
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

/// And the same for what a frame of the game answers rather than what a thread of orb's has done: frames
/// until the word or the player says `done` of the frame just run.
///
/// # Panics
/// After [`WAITS`], with the log.
fn frames_until(game: &Fake, what: &str, done: impl Fn() -> bool) {
    for _ in 0..WAITS {
        game.frame();
        if done() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!(
        "{what} did not happen in {WAITS} frame(s):\n  {}",
        game.log().lines().join("\n  ")
    );
}
