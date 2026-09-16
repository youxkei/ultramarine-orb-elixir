//! **The pad half of th07's own input read, which is orb's.**
//!
//! th07's read of a frame's buttons is one function, `GetInput` at 0x430b50: the keyboard through
//! `GetKeyboardState`, and then a tail call at 0x430f20 for the pad — the one DirectInput device the game
//! looks for and the one winmm joystick it falls back to. orb hooks that tail call and does not call
//! through — see `Th07`'s `GET_CONTROLLER_INPUT` — so everything a pad
//! does to the word the game acts on is orb's answer, out of the pads it samples on a thread of its own.
//!
//! **What is asserted here is the arithmetic itself**, which is what makes this file the other half of the
//! pad test in `th07.rs`: that one is that a pad the game has no device for reaches the word at all, and
//! this one is what reading one produces. The rules are the game's own, read off 0x430f20: the nine mapped
//! buttons through the table `th07.cfg` holds — shoot 0x1, bomb 0x2, focus 0x4, menu 0x8, the four
//! directions 0x10 to 0x80, skip 0x100, and a mapping entry below zero naming no button (0x43037d) — and
//! the stick against a quarter of the travel the pad's own caps report, either side of the middle.
//!
//! **This game does no pad arithmetic of its own**, which is what leaves the assertions here about orb: its
//! `input` builds the keyboard's word and hands it to orb's hook, the way the game's own read hands its tail
//! call the keys it read. So a bit that reaches the word reached it through orb.
//!
//! **And nothing here holds one button that is both shoot and focus.** th06 keeps a count of frames such a
//! button has been held and holds the player still from the eighth, which is
//! `the_pad_half_of_the_input_read.rs`'s subject; nothing in `Th07`'s reading of 0x430f20 names a count
//! like it, so whether this game has one is not established, and a test over it here would be a test of
//! orb's mapping and nothing of the game's.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A pad on the host, as an e2e test declares one: buttons enough for the game's own mapping to name any of
/// them, and an axis over a whole 16-bit travel — the same declaration `th07.rs`'s own pad test makes.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// The bits of th07's own input word, in the order its read fills them: read off the pad read at 0x430457
/// and 0x4305ac — see `Th07::pad_word`.
const SHOOT: u16 = 0x1;
const BOMB: u16 = 0x2;
const FOCUS: u16 = 0x4;
const MENU: u16 = 0x8;
const UP: u16 = 0x10;
const DOWN: u16 = 0x20;
const LEFT: u16 = 0x40;
const RIGHT: u16 = 0x80;
const SKIP: u16 = 0x100;

/// And which pad button each of those is with nobody's `th07.cfg` in the directory, which is the game's own
/// default: `0, 1, 2, 4, -1, -1, -1, -1, 3` — shoot, bomb, focus, menu and skip, with the four directions
/// naming no button. `UNMAPPED` is a button the default names nothing, so a bit it puts in the word would be
/// orb inventing a mapping.
const SHOOT_BUTTON: u32 = 0;
const BOMB_BUTTON: u32 = 1;
const FOCUS_BUTTON: u32 = 2;
const SKIP_BUTTON: u32 = 3;
const MENU_BUTTON: u32 = 4;
const UNMAPPED: u32 = 7;

/// Where a hat points, in the hundredths of a degree clockwise from straight up that a pad reports one in.
const HAT_UP: u32 = 0;

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

/// A quarter of the travel, which is the dead zone the game's own read makes of it either side of the middle.
fn a_quarter() -> u32 {
    PAD_TRAVEL.1 / 4
}

/// A launch with a pad on the host and orb's own thread having read it once, which is what says the sample
/// reaches the game at all: the pad is named in the log by the thread that found it.
fn with_a_pad(name: &str, settings: impl FnOnce(&mut orb_config::Config)) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            settings(config);
        },
    );
    let log = game.log();
    game.sim()
        .joystick()
        .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
    game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
    game
}

/// Frames until the word the game's own read answered is exactly `word`, waited for rather than counted: the
/// sample is taken on a thread nothing here drives, so what says a push has arrived is the word.
fn the_word_becomes(game: &Fake, what: &str, word: u16) {
    game.frames_until_a_thread(what, || game.word_the_input_read_answered() == word);
}

/// The buttons of the pad reach the word, each as the bit the game's own mapping names it by, and a button the
/// mapping names nothing for reaches it as nothing.
#[test]
fn the_buttons_of_a_pad_reach_th07s_word_through_the_games_own_mapping() {
    in_its_own_process(|| {
        let game = with_a_pad("th07-the-pad-buttons", |_| {});
        for (button, bit) in [
            (SHOOT_BUTTON, SHOOT),
            (BOMB_BUTTON, BOMB),
            (FOCUS_BUTTON, FOCUS),
            (SKIP_BUTTON, SKIP),
            (MENU_BUTTON, MENU),
        ] {
            game.sim().joystick().pushes(1 << button, centred());
            the_word_becomes(
                &game,
                &format!("button {button} reaching the word as {bit:#x} alone"),
                bit,
            );
        }

        // And a button the mapping names nothing for, pushed with one it does: the word is the one bit,
        // which is what says the other added nothing — a word that is *not* something cannot be waited for.
        game.sim()
            .joystick()
            .pushes((1 << SHOOT_BUTTON) | (1 << UNMAPPED), centred());
        the_word_becomes(
            &game,
            "the shot button reaching the word with an unmapped one beside it",
            SHOOT,
        );
        game.frames(8);
        assert_eq!(
            game.word_the_input_read_answered(),
            SHOOT,
            "a button the game's mapping names nothing for reached the word",
        );
    });
}

/// And the stick is the four directions, each axis measured against a quarter of the pad's own travel: inside
/// that is a stick nobody is pushing.
#[test]
fn the_stick_of_a_pad_reaches_th07s_word_past_a_quarter_of_its_travel() {
    in_its_own_process(|| {
        let game = with_a_pad("th07-the-pad-stick", |_| {});

        // Inside the dead zone, with the shot button held so the sample's arrival can be waited for: the word
        // is the button and no direction.
        game.sim()
            .joystick()
            .pushes(1 << SHOOT_BUTTON, centred() + a_quarter() - 1);
        the_word_becomes(&game, "a stick inside its own dead zone", SHOOT);

        // Past it, down the screen and up: Y grows downwards, the way the game's own read has it.
        game.sim().joystick().pushes(0, centred() + a_quarter() + 1);
        the_word_becomes(&game, "the stick pushed down", DOWN);
        game.sim().joystick().pushes(0, centred() - a_quarter() - 1);
        the_word_becomes(&game, "the stick pushed up", UP);

        // And the other axis, which a menu never reads and a player is moved by.
        game.sim()
            .joystick()
            .pushes_at(0, 0, centred() - a_quarter() - 1, centred());
        the_word_becomes(&game, "the stick pushed left", LEFT);
        game.sim()
            .joystick()
            .pushes_at(0, 0, centred() + a_quarter() + 1, centred());
        the_word_becomes(&game, "the stick pushed right", RIGHT);

        // Let go, and the word is empty again: what is added is what the stick says now.
        game.sim().joystick().pushes_at(0, 0, centred(), centred());
        the_word_becomes(&game, "the stick let go", 0);
    });
}

/// A pad let go of leaves the word the keyboard read made it: the pad's half is added to the keys' and takes
/// nothing of them away.
#[test]
fn a_pad_let_go_of_leaves_th07s_word_as_the_keyboard_read_it() {
    in_its_own_process(|| {
        let game = with_a_pad("th07-the-pad-let-go", |_| {});
        // The shot key held on the keyboard throughout, and the bomb button on the pad beside it.
        game.keyboard().set(orb_sim::keys::Z, true);
        game.sim().joystick().pushes(1 << BOMB_BUTTON, centred());
        the_word_becomes(&game, "the keys and the pad together", SHOOT | BOMB);

        game.sim().joystick().pushes(0, centred());
        the_word_becomes(&game, "the pad let go of", SHOOT);
        game.frames(8);
        assert_eq!(
            game.word_the_input_read_answered(),
            SHOOT,
            "a pad let go of took the keyboard's own half out of the word",
        );
        game.keyboard().set(orb_sim::keys::Z, false);
    });
}

/// The hat is the one thing on a pad the game reads on no device at all, which is why it is behind
/// `dpad_moves` where everything above is not: what the rest of this file asserts is what the game would have
/// produced from the same pad, and a d-pad is what it would not.
#[test]
fn the_setting_off_leaves_only_the_hat_out_of_th07s_word() {
    in_its_own_process(|| {
        let game = with_a_pad("th07-the-pad-dpad-off", |config| config.dpad_moves = false);
        let log = game.log();
        assert!(
            log.said("joystick: the d-pad is left as the game has it"),
            "the launch did not say the d-pad does nothing:\n  {}",
            log.lines().join("\n  ")
        );

        // The hat pushed up with the shot button held: the button reaches the word, and the hat does not.
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.sim().joystick().pushes_the_hat(HAT_UP);
        the_word_becomes(&game, "the shot button reaching the word", SHOOT);
        for _ in 0..30 {
            game.frame();
            assert_eq!(
                game.word_the_input_read_answered() & UP,
                0,
                "the d-pad reached the word in a launch that turned it off",
            );
        }
    });
}

/// And what that read costs reaches the `perf:` line, which is what the span orb's hook records is for: the
/// pad half is a tail call inside the game's own read and cannot be told apart from outside it.
#[test]
fn the_span_th07s_pad_read_costs_reaches_the_perf_line() {
    in_its_own_process(|| {
        let game = with_a_pad("th07-the-pad-perf-line", |_| {});
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
        assert!(
            perf.contains("input="),
            "the perf line has a joystick span and no input span to be inside: {perf}",
        );
    });
}
