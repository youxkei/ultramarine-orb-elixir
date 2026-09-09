//! The pad answering the questions orb puts over th07 — the mode at its title menu, and the retry menu a death
//! offers.
//!
//! Its own file rather than a case in `the_mode_question_over_th07.rs`, because what it reaches is a different half
//! of orb: those press keys, which orb reads for itself, and this pushes a *controller*, which orb's menus do not
//! read at all. `Game::pad` is what they read, and it is the game's to answer.
//!
//! **Which for this game is the pad orb sampled.** th07 looks for a DirectInput device of its own and falls back
//! to one winmm joystick, and the machine this was measured on has neither of those find the pad — see
//! `Th07`'s own `GET_CONTROLLER_INPUT`. So the pad that plays the game is the one orb samples and adds to the
//! game's own word, and a menu of orb's that read anything else would be a menu ignoring a pad that plainly works.
//!
//! What that costs if it is wrong is the whole mode on a pad: the retry menu is answered dozens of times in a
//! fight, and a hand holding a controller cannot reach for the keyboard for each of them.

use crate::fake::th07::Fake;
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::title;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A pad on the host, as an e2e test declares one: buttons enough for the game's own mapping to name any of them,
/// and an axis over a whole 16-bit travel — the same declaration `th07.rs`'s own pad test makes.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// The buttons th07's own default mapping calls shoot and bomb, which are the two orb's menus read as decide and
/// as back: the first and the second of `0, 1, 2, 4, -1, -1, -1, -1, 3` — see `Th07`'s `PAD_MAPPING`.
const SHOOT_BUTTON: u32 = 0;
const BOMB_BUTTON: u32 = 1;

/// Whether the pad orb last sampled has exactly these buttons down, which is the reading orb's own menus are
/// answered out of — see `orb_core::joystick::reading`.
fn sampled(buttons: u32) -> bool {
    orb_core::joystick::reading().is_some_and(|reading| reading.buttons == buttons)
}

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

/// A launch with a pad on the host and the mode question up over the title menu.
///
/// Put up on a keypress, which is the only way it goes up: the press orb holds back is the game's own, and a pad's
/// decide is one of the buttons that press is made of.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.sim()
        .joystick()
        .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
    game.frames_until("an overlay", 8, || log.said("overlay: ready"));
    // And the device orb found named, which is what says which of the two branches this pad is on: winmm's
    // joystick 0, sampled on a thread of orb's own, is the one th07 falls back to and the only one it has on the
    // machine `GET_CONTROLLER_INPUT` was measured on. See `a_pad_the_game_has_no_device_for.rs` for the sockets.
    game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
    game.image().the_front_end_is_up();
    game.image().the_title_menu_points_at_a_run();
    game.image().the_screen_has_been_up_for(60);
    game.one_frame();
    game.press_until(keys::Z, "the question", || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run, LANGUAGE)).is_empty(),
        "the press did not put the question up:\n  {}",
        log.lines().join("\n  ")
    );
    game.frames(READS_KEYS_AFTER);
    game
}

/// How many outcomes orb has reported for the question, counted rather than looked for: a line once written stays
/// written.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// The pad answers the mode question, and the log says which hand did.
///
/// The game is frozen on these frames, so its own reading of the pad is not running: what answers is orb asking
/// the game what the pad is doing, out of the reading orb sampled on a thread of its own.
#[test]
fn the_pad_answers_th07s_mode_question_and_is_named_as_the_hand_that_did() {
    in_its_own_process(|| {
        let game = asking("th07-pad-answers");
        let log = game.log();
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until("the pad's answer", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: answered on the pad"),
            "the answer was not named as the pad's:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("mode: pointdevice, was pointdevice"),
            "the pad chose no mode:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the stick and the hat both move its cursor, which is what makes the other mode reachable on a pad.
#[test]
fn the_stick_and_the_hat_both_move_th07s_mode_question() {
    in_its_own_process(|| {
        let game = asking("th07-pad-moves");
        let log = game.log();

        // Down on the stick, past the dead zone the pad's own travel makes, and then let go: the cursor is on the
        // other mode, which is what answering with it says.
        game.sim()
            .joystick()
            .pushes(0, centred() + PAD_TRAVEL.1 / 4 + 1);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(0, centred());
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until("the pad's answer", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the stick did not move the cursor to the other mode:\n  {}",
            log.lines().join("\n  ")
        );

        // And the hat the other way, on the question the next press puts up: the cursor starts where the last
        // answer left it, so up is back to the mode with chapters.
        game.sim().joystick().pushes(0, centred());
        // The title menu again, with the cursor back on a run. The press orb handed on for the first answer
        // lands on the read after the answer and is one this game's title acts on, so a frame goes first for it
        // to land in — the front end walking to the difficulty select — and the title is put back after it; then
        // a frame with the cursor there before the press, for the reason `Fake::answers_the_mode` gives.
        game.frame();
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        game.frame();
        let from = log.written();
        game.press_until(keys::Z, "the question again", || {
            log.said_since(from, "menu: Run is under the cursor, asking which mode")
        });
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes_the_hat(0);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(0, centred());
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until("the pad's second answer", 60, || outcomes(&game) == 2);
        assert!(
            log.said("mode: pointdevice, was normal"),
            "the hat did not move the cursor back:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the button the game reads as back cancels it, which is the answer that starts no run at all.
#[test]
fn the_pads_back_button_cancels_th07s_mode_question() {
    in_its_own_process(|| {
        let game = asking("th07-pad-cancels");
        let log = game.log();
        game.sim().joystick().pushes(1 << BOMB_BUTTON, centred());
        game.frames_until("the pad's cancel", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: not chosen on the pad"),
            "the pad's back button did not cancel:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !game.state().in_run,
            "a run was started by a question the pad cancelled",
        );
    });
}

/// And the retry menu a death offers, which is the menu a pad has to reach most: answered on the pad, with the
/// chapter put back.
#[test]
fn the_pad_answers_th07s_retry_menu() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-pad-retries",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.sim()
            .joystick()
            .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        let at_the_chapter = game.state();

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until("the retry menu answered on the pad", 60, || {
            log.said("retry: the chapter again on the pad")
        });
        assert_eq!(
            game.state(),
            at_the_chapter,
            "the chapter the pad asked for is not where the run came back to",
        );
    });
}

/// A pad button held from before the question went up is not a press when the frames it reads nothing over run
/// out. The pad is read every frame, those frames included, which is what makes the edge come out right — the
/// shot button is what chose the item this is asked over.
///
/// **In this game the pad's shot button and the keyboard's shot key are one bit of one word**, the pad half
/// being added to the keys' — see `the_pad_half_of_th07s_input_read.rs` — so the pad's push is itself the press
/// that puts the question up, where th06's e2e test has the keyboard put it up over a pad already held. What is
/// the same in both is what follows: the push held on from there is the press that opened the question, spent,
/// and not an answer however long it is held.
#[test]
fn a_button_held_from_before_th07s_question_does_not_answer_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-pad-held",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.sim()
            .joystick()
            .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        // A frame with the cursor there before the push, for the reason `Fake::answers_the_mode` gives.
        game.frame();

        // Pushed and never let go: the push is the edge the question goes up on, and the push held on from there
        // is the press that opened it — spent, and not an answer however long it is held.
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until_a_thread("the question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        for _ in 0..READS_KEYS_AFTER + 60 {
            game.frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a pad button held from before the question answered it:\n  {}",
                log.lines().join("\n  ")
            );
        }

        // Let it go and push it again, which is what somebody actually does.
        //
        // **Waited for on the sample orb's own menus read**, and not on the word the game is handed: while a
        // question is up the game is handed nothing at all, so the word cannot say whether the thread has read
        // the release yet.
        game.sim().joystick().pushes(0, centred());
        game.frames_until_a_thread("the button let go of", || sampled(0));
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until("the pad's answer", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: answered on the pad"),
            "the press made after the hold was not the pad's answer:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
