//! The pad answering the question orb puts over the game's own title menu.
//!
//! Its own file rather than a case in `mode_question.rs`, because what it reaches is a different half
//! of orb: those press keys, which orb reads for itself, and this pushes a *controller*, which orb
//! does not read at all. `Game::pad` asks the game — the device the game polls, through the mapping
//! the game keeps — and the reason is measured on this machine and in `DONE.md`: a pad in XInput's
//! second slot is one DirectInput has and winmm's joystick 0 does not, so a menu of orb's driven from
//! orb's own sample answers to a pad the game has not got. Which looked exactly like orb's menus
//! ignoring a pad that plainly worked.
//!
//! So the game here has a controller: an object and a vtable in its own memory, with the addresses of
//! three real functions in the slots the game's read calls through, the same way its memory holds the
//! address of the Direct3D device orb draws through. What that covers, and nothing covered before, is
//! `Th06::controller_pad` itself — the poll, the read, the buttons out of the device's own array, the
//! stick against the threshold the game keeps, and the mapping that says which button is which.

mod fake;

use fake::{Fake, MAPPING, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::RunStart;
use orb_core::game::th06::image::{Pushed, Screen};
use orb_core::menu::By;
use orb_core::mode::{Mode, title};
use orb_sim::keys;

fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// A game at its title menu with the question up over it, and nothing on the pad yet.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    // Put up on a keypress, which is the only way it goes up: the press orb holds back is the game's
    // own, and a pad's decide is one of the buttons that press is made of.
    game.press(keys::Z);
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run)).is_empty(),
        "the press did not put the question up: {:?}",
        game.log().lines(),
    );
    game.frames(READS_KEYS_AFTER);
    game
}

/// How many outcomes orb has reported for the question, counted rather than looked for: a line once
/// written stays written, and a scenario that asks twice needs to tell the second answer from the
/// first.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// The pad answers it, and the log says which hand did: the game is frozen on these frames, so its own
/// reading of the pad is not running, and without orb asking the game for one a pad does nothing at
/// all here while working perfectly on the game's own menu a keypress earlier.
#[test]
fn the_pad_answers_and_is_named_as_the_hand_that_did() {
    in_its_own_process(|| {
        let game = asking("pad-answers");
        game.push(Pushed::button(MAPPING.shoot));
        game.frames_until("the pad's answer", 8, || outcomes(&game) == 1);
        assert!(
            game.log()
                .said(&format!("mode: answered on the {}", By::Pad)),
            "the answer was not named as the pad's: {:?}",
            game.log().lines(),
        );
        assert!(
            game.log().said("mode: pointdevice, was pointdevice"),
            "the pad chose no mode: {:?}",
            game.log().lines(),
        );
        // And the press it stands in for reaches the game's own menu, which is what starts the run.
        game.frames_until("the run being asked for", 90, || {
            game.image().front_end_now().screen == Screen::ShotType
        });
    });
}

/// A pad button held from before the question went up is not a press when the frames it reads nothing
/// over run out. The pad is read every frame, those frames included, which is what makes the edge come
/// out right — the shot button was held to choose the item this is asked over.
#[test]
fn a_button_held_from_before_the_question_does_not_answer_it() {
    in_its_own_process(|| {
        let game = Fake::attach("pad-held", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.at_the_title_menu();
        // Held before the question is even asked for, and never let go.
        game.push(Pushed::button(MAPPING.shoot));
        game.press(keys::Z);
        game.frames_until("the question", 8, || {
            !game.says(title(Menu::Run)).is_empty()
        });
        for _ in 0..READS_KEYS_AFTER + 60 {
            game.frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a pad button held from before the question answered it: {:?}",
                game.log().lines(),
            );
        }

        // Let it go and push it again, which is what somebody actually does.
        game.push(Pushed::none());
        game.frame();
        game.push(Pushed::button(MAPPING.shoot));
        game.frames_until("the pad's answer", 8, || outcomes(&game) == 1);
    });
}

/// The pad moves the cursor too, so 完全無欠モード is reachable without touching the keyboard — on the
/// stick and on the hat, which the game reads out of different fields of its device.
#[test]
fn the_stick_and_the_hat_both_move_the_cursor() {
    in_its_own_process(|| {
        let game = asking("pad-moves");
        // Down to レガシーモード on the stick: past the threshold the game keeps beside its mapping, since
        // the middle of an axis is not a direction.
        game.push(Pushed {
            y: i32::from(MAPPING.y_axis) + 1,
            ..Pushed::none()
        });
        game.frames(2);
        game.push(Pushed::none());
        game.frames(2);
        assert_eq!(
            under_the_cursor(&game),
            Mode::Normal,
            "the stick did not move the cursor",
        );

        // And back up on the hat, which reports a direction rather than a distance.
        game.push(Pushed {
            hat: 0,
            ..Pushed::none()
        });
        game.frames(2);
        game.push(Pushed::none());
        game.frames(2);
        assert_eq!(
            under_the_cursor(&game),
            Mode::Pointdevice,
            "the hat did not move the cursor",
        );

        // Chosen there, on the pad's own decide.
        game.push(Pushed::button(MAPPING.shoot));
        game.frames_until("the pad's answer", 8, || outcomes(&game) == 1);
        assert!(
            game.log().said("mode: pointdevice, was pointdevice"),
            "the pad chose the mode the cursor had left: {:?}",
            game.log().lines(),
        );
    });
}

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for (mode, text) in orb_core::mode::CHOICES {
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == orb::menu_ui::SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        }
    }
    on.expect("neither mode is lit")
}
