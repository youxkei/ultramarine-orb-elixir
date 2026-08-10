//! The pad answering that same question where the game has **no DirectInput device**, which is the branch
//! it asks winmm on.
//!
//! Its own file rather than a case in `mode_on_the_pad.rs`, because it is the other device
//! entirely. `Controller::GetControllerInput` asks winmm for joystick 0 *only* where its own
//! `EnumDevices(DI8DEVCLASS_GAMECTRL, DIEDFL_ATTACHEDONLY)` found nothing attached; where it found
//! something it polls that through DirectInput and never asks winmm at all. So which of the two a menu of
//! orb's reads has to be the same one the game is on, and that one is here.
//!
//! **This is also the path that used to work and must not have been broken.** What
//! `orb_core::joystick` was measured on is this one: with nothing plugged in that call answers
//! `JOYERR_PARMS` after a large part of a frame and spends nearly all of it on the CPU, which was the
//! whole of the frame-pacing trouble and is why the read moved onto a thread of orb's own. What the game
//! is handed is the last sample that thread took.
//!
//! What the laid-out game brings is the read and nothing more: it asks winmm once a frame through orb's
//! replacement of that import entry, the way its own `GetControllerInput` does. The host's pad is
//! `orb_sim::Joystick`, and it is the host's because that is whose device winmm's joystick 0 is.

use crate::fake::th06::{Fake, MAPPING, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th06::image::Screen;
use orb_core::menu::By;
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// The pad this plugs in, as an Xbox One pad answers `joyGetDevCapsA` on this machine: sixteen buttons,
/// and a Y axis over the whole of a 16-bit travel.
const BUTTONS: u32 = 16;
const TRAVEL: (u32, u32) = (0, 65535);

/// The name the log says that device is, which is what an e2e test waits for the sample by: the read
/// happens on a thread of orb's own, so what says it has happened is the line that thread writes.
const NAMED: &str = "joystick: mid=045e pid=02ff";

/// What the thread asks the host to wait between two reads, in milliseconds: four while a pad is
/// answering, which is four samples a frame, and a second while none is — the read costing most of a
/// frame of CPU with nothing plugged in being the whole reason it is off the game's thread.
const SAMPLED_MS: u32 = 4;
const LOOKED_FOR_MS: u32 = 1000;

/// How long an e2e test gives that thread, in frames with a millisecond of real time each.
///
/// **Real time and not simulated**, which is the one place an e2e test here needs any: the sample is taken
/// on a thread, that being the whole point of it, so what is being waited for is that thread being
/// scheduled and not a number of the game's frames. It is orders of magnitude more than the first read
/// takes — that one being the slow one, winmm's joystick support coming up — and a wait that runs out is
/// a failure naming what it was waiting for.
const WAITS: u32 = 3000;

/// And the span the game's own read costs reaches the `perf:` line, which is the whole of what orb's hook
/// over `Controller::GetControllerInput` is for.
///
/// That read is a tail call inside `Controller::GetInput` and cannot be told apart from outside it, so
/// timing it means standing in front of it — which is what the hook does and the only thing it does. What
/// this file's own head records about that read is what that line said, and a launch whose perf line has
/// no joystick span in it is a launch where nobody could have found it out.
///
/// Verbose, because that is the gate: a real launch installs this hook only at Verbose and leaves the read
/// untimed otherwise. A game that hands the function over cannot be gated by an installation — it has one
/// call site whichever way the launch was configured — so the gate is a flag orb sets at the attach, and this
/// is the launch that asks for it.
#[test]
fn the_span_the_games_own_joystick_read_costs_reaches_the_perf_line() {
    in_its_own_process(|| {
        let game = Fake::attach("a-winmm-pad-the-perf-line", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.sim().joystick().attach(BUTTONS, TRAVEL.0, TRAVEL.1);
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

/// A game whose own enumeration found no controller, with a pad on the host, sitting at its title menu
/// with the question up and orb's own thread having sampled that pad at least once.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.image().no_controller();
    game.sim().joystick().attach(BUTTONS, TRAVEL.0, TRAVEL.1);
    game.at_the_title_menu();
    waits_for_the_sample(&game);
    // Put up on a keypress, which is the only way it goes up.
    game.press(keys::Z);
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run)).is_empty(),
        "the press did not put the question up:\n  {}",
        game.log().lines().join("\n  ")
    );
    game.frames(READS_KEYS_AFTER);
    game
}

/// Runs frames until orb's own thread has read the pad, which is what its menus are answered out of.
///
/// The frames go on meanwhile because the game's own read is what starts that thread: the first
/// `joyGetPosEx` of the run is the startup check, and orb spawns on it.
///
/// # Panics
/// After [`WAITS`], with the log: an e2e test waiting for a sample that never came is about to assert on a
/// pad nothing read.
fn waits_for_the_sample(game: &Fake) {
    waits_until(game, "orb's own thread reading the pad", || {
        game.log().said(NAMED)
    });
}

/// And the same wait for anything a push has to travel through that thread to reach: frames of the game,
/// a millisecond of real time each, until `done`.
///
/// Which is why an e2e test here cannot simply push and run two frames the way the one about the
/// controller does: that pad is the game's own memory and orb reads it where it stands, and this one is
/// sampled every four milliseconds by a thread nothing here drives.
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

/// A pad winmm has answers the question, and the axis it is read through is measured against the caps of
/// the device that answered.
///
/// **The calibration is the half that cannot be left to the game.** `GetControllerInput` places the centre
/// of each axis at `(wXmin + wXmax) / 2` with a dead zone of a quarter of the travel, read out of the
/// `JOYCAPSA` at 0x69d760 — an address that appears exactly once in the whole exe, in the
/// `joyGetDevCapsA` call the startup check makes, and that check only reads it where a joystick answered
/// `joyGetPosEx` first. So a pad that turns up after that is measured against zeros: its centred axes,
/// 32767 of a 65535 travel, both read as far over, and the game spends the rest of the run with two
/// directions held. Watched happening on this machine, and watched not happening once the caps were
/// handed over with the sample they belong to.
#[test]
fn a_pad_winmm_has_answers_the_question_and_its_axis_is_measured_against_its_own_caps() {
    in_its_own_process(|| {
        let game = asking("winmm-pad-answers");

        // The game's own copy of the caps is this device's, written from the sample rather than left as
        // the startup check found it.
        assert!(
            game.log()
                .said("joystick: the game's axis calibration was not this device's"),
            "the caps were not handed over to the game:\n  {}",
            game.log().lines().join("\n  ")
        );

        // Down to レガシーモード on the stick, past the dead zone the caps make: a quarter of the travel
        // either side of the middle, which is what the game's own read does with the same two numbers.
        game.sim()
            .joystick()
            .pushes(0, centred() + TRAVEL.1 / 4 + 1);
        waits_until(&game, "the stick moving the cursor", || {
            under_the_cursor(&game) == Mode::Normal
        });
        game.sim().joystick().pushes(0, centred());

        // And back up on the hat, which is where a d-pad reports: hundredths of a degree clockwise from
        // straight up, and the axes say nothing about it.
        game.sim().joystick().pushes_the_hat(STRAIGHT_UP);
        waits_until(&game, "the hat moving the cursor", || {
            under_the_cursor(&game) == Mode::Pointdevice
        });
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);

        // Chosen on the pad's own decide, which is the button the game's mapping calls shoot.
        game.sim().joystick().pushes(1 << MAPPING.shoot, centred());
        waits_until(&game, "the pad's answer", || outcomes(&game) == 1);
        assert!(
            game.log()
                .said(&format!("mode: answered on the {}", By::Pad)),
            "the answer was not named as the pad's:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.log().said("mode: pointdevice, was pointdevice"),
            "the pad chose the mode the cursor had left:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the press it stands in for reaches the game's own menu, which is what starts the run.
        game.frames_until("the run being asked for", 90, || {
            game.image().front_end_now().screen == Screen::ShotType
        });
    });
}

/// A device that answers with no buttons and no axes is not a pad, however much it answers.
///
/// Windows can leave exactly one of these on joystick 0 whenever the pad it has sits in XInput's second
/// slot: a device answering `joyGetPosEx` successfully with every field zero. Measured by asking all
/// three interfaces at once — winmm has the phantom at index 0 and nothing on the rest, while
/// DirectInput and XInput both have the real pad. Believing the phantom costs a line in the log claiming
/// a pad answered, and the game's axis calibration written from a device that has no axes.
#[test]
fn a_device_with_no_buttons_and_no_axes_drives_nothing() {
    in_its_own_process(|| {
        let game = Fake::attach("winmm-pad-phantom", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.sim().joystick().attach_a_phantom();
        game.at_the_title_menu();
        for _ in 0..WAITS {
            if game.log().said("which is no pad") {
                break;
            }
            game.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(
            game.log().said(
                "joystick 0 is mid=413d pid=2104 \"USB Gaming Controller\" with no buttons and no \
                 axes, which is no pad; orb's own menus will not be driven from it"
            ),
            "orb did not say what answered:\n  {}",
            game.log().lines().join("\n  ")
        );
        // And nothing of the game's own calibration was written from it, which is the cost of believing
        // one: axes it has not got, with the game measuring against them for the rest of the run.
        assert!(
            !game
                .log()
                .said("joystick: the game's axis calibration was not this device's"),
            "the game's axes were calibrated from a device with none:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The question up, and every button on that device pushed: it answers nothing, because it is not
        // a pad.
        game.press(keys::Z);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(u32::MAX, 0);
        for _ in 0..READS_KEYS_AFTER {
            game.frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a device with no buttons and no axes answered the question:\n  {}",
                game.log().lines().join("\n  ")
            );
        }
    });
}

/// Nothing plugged in is the case this whole module exists for, and a pad that turns up later is picked up
/// within the second the thread waits.
///
/// **The empty socket is the expensive one.** Measured: with nothing there `joyGetPosEx` answers
/// `JOYERR_PARMS` after a large part of a frame and spends nearly all of it on the CPU rather than
/// waiting, which was the whole of the frame-pacing trouble and is why the read is on a thread at all. A
/// pad *attached* answers the same call fast.
///
/// So the thread asks once a second while nothing answers rather than four times a frame, which is what
/// bounds how late a pad plugged in mid-session is picked up. Watched happening: a run that started with
/// the pad asleep — the first read the slow one, winmm's joystick support coming up — had it wake
/// mid-run, took the calibration on the next frame, and was then driven through the menus with nothing
/// drifting.
#[test]
fn an_empty_socket_is_named_and_a_pad_that_turns_up_later_drives_the_menu() {
    in_its_own_process(|| {
        let game = Fake::attach("winmm-pad-none", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.at_the_title_menu();
        waits_until(&game, "orb saying the socket is empty", || {
            game.log().said("joystick: there is no joystick 0")
        });

        // A second between reads, and never the four milliseconds a pad gets: the ask goes through the
        // seam, so the cadence is a number an e2e test reads back rather than a wait it sits through.
        //
        // What is in the record and not what all of it is. The wait is never sooner than the read itself
        // took, and here one clock serves both threads — a read the game's own frames advanced it across
        // is charged that time — so a longer ask can stand between two of these. That a *shorter* one
        // cannot is the assertion: with nothing answering the wait is `max(1000, cost)`.
        waits_until(
            &game,
            "the second the thread waits for an empty socket",
            || game.sim().clock().sleeps().contains(&LOOKED_FOR_MS),
        );
        assert!(
            !game.sim().clock().sleeps().contains(&SAMPLED_MS),
            "the empty socket was read at a pad's cadence: {:?}",
            game.sim().clock().sleeps(),
        );

        // And that thread runs below the game's, which is the other half of costing the frame nothing: a
        // sample a millisecond old costs nobody anything and a late frame costs the thing orb was written
        // for. One thread and not this one — the frame's own is what it is being put below.
        let below = game.sim().threads_below_normal();
        assert_eq!(below.len(), 1, "{below:?} put themselves below the game's");
        assert_ne!(
            below[0],
            orb_api::thread::current_id(),
            "the thread the game's frames run on put itself below the game's own priority",
        );

        // The question up, and nothing to answer it with: a menu of orb's reads the sample, and there is
        // no pad in it.
        game.press(keys::Z);
        game.one_frame();
        assert!(
            !game.says(title(Menu::Run)).is_empty(),
            "the press did not put the question up:\n  {}",
            game.log().lines().join("\n  ")
        );
        game.frames(READS_KEYS_AFTER);
        for _ in 0..READS_KEYS_AFTER {
            game.frame();
            assert_eq!(
                outcomes(&game),
                0,
                "the question was answered with nothing plugged in:\n  {}",
                game.log().lines().join("\n  ")
            );
        }

        // And then a pad, which the thread finds on its next read — within the second it waits while
        // nothing is answering, which on a simulated host is a second asked for rather than a second
        // spent.
        game.sim().joystick().attach(BUTTONS, TRAVEL.0, TRAVEL.1);
        waits_until(&game, "the pad that turned up being read", || {
            game.log().said(NAMED)
        });
        // And the cadence changes with it: four samples a frame, so that what the game is handed is never
        // a frame behind what the stick is doing.
        waits_until(&game, "the cadence of a pad that is answering", || {
            game.sim().clock().sleeps().contains(&SAMPLED_MS)
        });
        game.sim().joystick().pushes(1 << MAPPING.shoot, centred());
        waits_until(&game, "the pad's answer", || outcomes(&game) == 1);
        assert!(
            game.log()
                .said(&format!("mode: answered on the {}", By::Pad)),
            "the pad that turned up was not the hand that answered:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Where the Y axis sits with nothing touching it: halfway along the travel the caps name.
fn centred() -> u32 {
    TRAVEL.0 + (TRAVEL.1 - TRAVEL.0) / 2
}

/// The hat pushed straight up, which winmm reports as nothing clockwise of it.
const STRAIGHT_UP: u32 = 0;

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for (mode, text) in orb_core::mode::CHOICES {
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == orb_core::menu_ui::SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        }
    }
    on.expect("neither mode is lit")
}
