//! The pad answering the questions orb puts over th10 — the mode at its main menu, and the retry menu a death
//! offers — what `the_mode_of_th07_on_the_pad` asks of that game, asked of this one.
//!
//! **The game is frozen on these frames**, so its own read of the pad (0x44a190) is not running: what answers is
//! orb asking the game what the pad is doing, `Game::pad`, out of the reading orb sampled on a thread of its own
//! and the device the game holds. **And which button is which is the game's mapping**, the nine buttons
//! `th10.cfg` names from its `+0x4`, laid at `+0x58` of the player's input (0x4208ff through 0x420934): the shot
//! is decide and the bomb or the pause is cancel, as every menu of the game takes `0x1001` and `0xa`.
//!
//! What that costs if it is wrong is the whole mode on a pad: the retry menu is answered dozens of times in a
//! fight, and a hand holding a controller cannot reach for the keyboard for each of them.

use crate::fake::th10::Fake;
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th10::image::{STEP_READING, Screen};
use orb_core::mode::title;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A pad on the host, as an e2e test declares one: buttons enough for the game's own mapping to name any of them,
/// and an axis over a whole 16-bit travel — the declaration th07's file makes.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// **The buttons th10's own mapping calls the shot, the bomb and the pause with no `th10.cfg` read over it**: the
/// first, the second and the fourth of `0, 1, 2, 3, -1, -1, -1, -1, 4` in the exe's own `.data` at `0x474e88`,
/// which 0x418c40 writes into a configuration the file does not answer.
const SHOT_BUTTON: u32 = 0;
const BOMB_BUTTON: u32 = 1;
const PAUSE_BUTTON: u32 = 3;

/// How long the laid-out stage lasts here, which is longer than these play.
const A_STAGE: u32 = 100_000;

/// How many frames a screen is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

/// Whether the title is on `screen` and reading a key.
fn reading(game: &Fake, screen: Screen) -> bool {
    let (on, step, _, _) = game.image().where_the_title_is();
    game.image().the_title_is_up() && on == screen as i32 && step == STEP_READING
}

/// A launch with a pad on the host, orb's own thread having named it.
fn with_a_pad(name: &str) -> Box<Fake> {
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
    game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
    game
}

/// And the mode question up over the main menu, put up on a keypress, which is the only way it goes up.
fn asking(name: &str) -> Box<Fake> {
    let game = with_a_pad(name);
    let log = game.log();
    game.frames_until(
        "the title's screen that waits for a key",
        FRAMES_FOR_A_SCREEN,
        || reading(&game, Screen::PressAKey),
    );
    game.press(keys::Z);
    game.frames_until("the main menu", FRAMES_FOR_A_SCREEN, || {
        reading(&game, Screen::MainMenu)
    });
    game.press_until(keys::Z, "the question", || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game.one_frame();
    assert_eq!(
        game.says(title(Menu::Run, LANGUAGE)).len(),
        1,
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

/// The pad's shot button answers the mode question, and the log says which hand did.
#[test]
fn the_pad_answers_th10s_mode_question_and_is_named_as_the_hand_that_did() {
    in_its_own_process(|| {
        let game = asking("th10-pad-answers");
        let log = game.log();
        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
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

/// And the stick moves its cursor, past the quarter of its travel this game's own read counts a push from
/// (0x44a2b8 through 0x44a2d5) — which is what makes the other mode reachable on a pad.
#[test]
fn the_stick_moves_th10s_mode_question() {
    in_its_own_process(|| {
        let game = asking("th10-pad-stick");
        let log = game.log();
        game.sim()
            .joystick()
            .pushes(0, centred() + PAD_TRAVEL.1 / 4 + 1);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(0, centred());
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
        game.frames_until("the pad's answer", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the stick did not move the cursor to the other mode:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the hat — a d-pad — moves it too, which this game's own read has no look at: a menu of orb's is orb's to
/// read, and a d-pad is where a thumb on a pad goes for a list.
#[test]
fn the_hat_moves_th10s_mode_question() {
    in_its_own_process(|| {
        let game = asking("th10-pad-hat");
        let log = game.log();
        game.sim().joystick().pushes_the_hat(18_000);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
        game.frames_until("the pad's answer", 60, || outcomes(&game) == 1);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the hat did not move the cursor to the other mode:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// The question cancelled on the pad's `button`.
fn cancelled_on(name: &str, button: u32) {
    let game = asking(name);
    let log = game.log();
    game.sim().joystick().pushes(1 << button, centred());
    game.frames_until("the pad's cancel", 60, || outcomes(&game) == 1);
    assert!(
        log.said("mode: not chosen on the pad"),
        "the pad's button {button} did not cancel:\n  {}",
        log.lines().join("\n  ")
    );
}

/// And the bomb button cancels it, which is one of the two the game's own menus take as cancel.
#[test]
fn the_pads_bomb_button_cancels_th10s_mode_question() {
    in_its_own_process(|| cancelled_on("th10-pad-cancels-on-the-bomb", BOMB_BUTTON));
}

/// And the pause button, which is the other.
#[test]
fn the_pads_pause_button_cancels_th10s_mode_question() {
    in_its_own_process(|| cancelled_on("th10-pad-cancels-on-the-pause", PAUSE_BUTTON));
}

/// And the retry menu a death offers, which is the menu a pad has to reach most: answered on the pad, with the
/// chapter put back.
#[test]
fn the_pad_answers_th10s_retry_menu() {
    in_its_own_process(|| {
        let game = with_a_pad("th10-pad-retries");
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 1"));
        game.frames(READS_KEYS_AFTER);
        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
        game.frames_until("the retry menu answered on the pad", 60, || {
            log.said("retry: the chapter again on the pad")
        });
        let state = game.state();
        assert_eq!(
            state.lives, 2,
            "the chapter did not put the run's life back"
        );
        assert!(state.playing, "the run was not left playing");
    });
}
