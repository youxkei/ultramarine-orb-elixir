//! **The two windows orb's rewrite of `CreateWindowExA` hands straight on, over a launch of th07.**
//!
//! `the_window_of_th07.rs` is every window orb *decides*: where it goes on the monitor, how big the client comes
//! out, and the black beside the game. This is the other side of the same rewrite — the asks it must not touch —
//! and there are exactly two of them, both settled before a rectangle is worked out at all:
//!
//! - **a window that is not the game's own class.** th07 registers its play window as `BASE`, which is what orb
//!   recognises one by; a window of any other class made the size orb decided for the play window would be a
//!   window nobody asked for. The class is looked at *first*, so such a window is not even the reason the host is
//!   asked about its panel.
//! - **a monitor that cannot be read.** Fullscreen is the monitor's rectangle and a size is centred on it, so a
//!   host that will not say where its panel is is a host with nothing to lay a window out against. The window is
//!   then the one the game asked for, which is the window every launch before orb had.
//!
//! What is read back is what the host was really asked for, out of `orb_sim::Windows::made` — the same instrument
//! `the_window_of_th07.rs` uses, and the only thing that can tell a rewrite from a pass-through.

use crate::fake::th07::{ASKED_AT, ASKED_SIZE, ASKED_STYLE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};
use orb_sim::Made;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The size the launches below are configured for, which is what makes the assertions say something: a window orb
/// laid out would be this size and centred, and every one of these is the game's own ask instead.
const CONFIGURED: Screen = Screen::Window {
    width: 1280,
    height: 720,
};

/// The one window the host was asked for.
///
/// # Panics
/// Where it was asked for none, or for more than one.
fn made(game: &Fake) -> Made {
    let made = game.sim().windows().made();
    assert_eq!(
        made.len(),
        1,
        "the host was asked for {} window(s), where this e2e test is about one",
        made.len(),
    );
    made[0]
}

/// The ask the game itself makes, which is what a pass-through leaves standing.
fn is_the_games_own(made: &Made) {
    assert_eq!(
        (made.asked.left, made.asked.top),
        ASKED_AT,
        "the position the game asked for was rewritten",
    );
    assert_eq!(
        (made.asked.width(), made.asked.height()),
        ASKED_SIZE,
        "the size the game asked for was rewritten",
    );
    // The style as the host reads it back, which is the one thing about the ask that is real: the game asks for a
    // caption and a system menu, and a window orb laid out fullscreen would have neither.
    assert!(made.framed, "the style the game asked for was rewritten");
    assert_eq!(
        ASKED_STYLE & orb_core::window::WINDOWED_STYLE,
        ASKED_STYLE,
        "the style this game asks for is not one that has a frame, so the assertion above says nothing",
    );
}

/// A window of another class is handed on with every argument the game gave it, and the monitor is not even read
/// for it.
#[test]
fn a_window_of_th07s_that_is_not_its_own_class_is_left_as_it_was_asked_for() {
    in_its_own_process(|| {
        // On a monitor, and configured for a size: everything orb needs to lay a window out is there, so the only
        // reason this ask comes through untouched is the class.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-another-class",
            Work::wandering(WORK_US),
            |config| {
                config.screen = CONFIGURED;
                config.log_level = LogLevel::Verbose;
            },
        );
        let reads = game.sim().windows().monitor_reads().len();

        game.creates_another_window();
        is_the_games_own(&made(&game));
        assert_eq!(
            game.sim().windows().monitor_reads().len(),
            reads,
            "the monitor was read for a window that is not the game's own",
        );
        // And nothing said about it: the line orb writes for the window it decided names a client and a size, and
        // a window it decided nothing about has none of that to say.
        assert!(
            !game.log().said("screen: window at"),
            "orb wrote its own window down for a window of another class:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And on a host that will not say where its panel is, the game's own window is what the game gets.
#[test]
fn a_monitor_that_cannot_be_read_leaves_th07s_window_as_it_asked_for_it() {
    in_its_own_process(|| {
        // No monitor declared, which is a host with none to read — see `orb_sim::Windows::new`. The size is
        // configured all the same, so what is missing is only the rectangle to centre it on.
        let game = Fake::attach_without_a_monitor(
            Display::agreed(120),
            "th07-no-monitor",
            Work::wandering(WORK_US),
            |config| {
                config.screen = CONFIGURED;
                config.log_level = LogLevel::Verbose;
            },
        );
        assert_eq!(
            game.sim().windows().monitor_now(),
            None,
            "this host has a monitor, so there is nothing for orb to fail to read",
        );

        game.creates_its_window();
        is_the_games_own(&made(&game));
        assert!(
            !game.log().said("screen: window at"),
            "orb laid a window out against a monitor it could not read:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
