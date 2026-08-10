//! **The two windows orb's rewrite of `CreateWindowExA` hands straight on.**
//!
//! `the_window.rs` is every window orb *decides*: where it goes on the monitor, how big the client comes
//! out, and the black beside the game. This is the other side of the same rewrite — the asks it must not
//! touch — and there are exactly two of them, both settled before a rectangle is worked out at all:
//!
//! - **a window that is not the game's own class.** The game makes others, and one of those made the size
//!   orb decided for the play window would be a window nobody asked for. The class is looked at *first*, so
//!   such a window is not even the reason the host was asked about its panel.
//! - **a monitor that cannot be read.** Fullscreen is the monitor's rectangle and a size is centred on it,
//!   so a host that will not say where its panel is is a host with nothing to lay a window out against. The
//!   window is then the one the game asked for, which is the window every launch before orb had.
//!
//! What is read back is what the host was really asked for, out of `orb_sim::Windows::made` — the same
//! instrument `the_window.rs` uses, and the only thing that can tell a rewrite from a pass-through.

use crate::fake::th06::{ASKED_AT, ASKED_SIZE, ASKED_STYLE, Fake, the_run};
use crate::fake::{Launched, Panel, in_its_own_process};
use orb_config::{LogLevel, Screen};
use orb_sim::Made;

/// The size the launch below is configured for, which is what makes the assertions say something: a window
/// orb laid out would be this size and centred, and every one of these is the game's own ask instead.
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
    // The style as the host reads it back, which is the one thing about the ask that is real: the game asks
    // for a caption and a system menu, and a window orb laid out fullscreen would have neither.
    assert!(made.framed, "the style the game asked for was rewritten",);
    assert_eq!(
        ASKED_STYLE & orb_core::window::WINDOWED_STYLE,
        ASKED_STYLE,
        "the style this game asks for is not one that has a frame, so the assertion above says nothing",
    );
}

/// A window of another class is handed on with every argument the game gave it, and the monitor is not even
/// read for it.
#[test]
fn a_window_that_is_not_the_games_own_class_is_left_as_it_was_asked_for() {
    in_its_own_process(|| {
        // On a panel, and configured for a size: everything orb needs to lay a window out is there, so the
        // only reason this ask comes through untouched is the class.
        let game = Fake::attach_to_a_panel(Panel::scaled(), "another-class", the_run(), |config| {
            config.screen = CONFIGURED;
            config.log_level = LogLevel::Verbose;
        });
        let reads = game.sim().windows().monitor_reads().len();

        game.creates_another_window();
        is_the_games_own(&made(&game));
        assert_eq!(
            game.sim().windows().monitor_reads().len(),
            reads,
            "the monitor was read for a window that is not the game's own",
        );
        // And nothing said about it: the line orb writes for the window it decided names a client and a
        // size, and a window it decided nothing about has none of that to say.
        assert!(
            !game.log().said("screen: window at"),
            "orb wrote its own window down for a window of another class:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And on a host that will not say where its panel is, the game's own window is what the game gets.
#[test]
fn a_monitor_that_cannot_be_read_leaves_the_window_as_the_game_asked_for_it() {
    in_its_own_process(|| {
        // No panel declared, which is a host with no monitor to read — see `orb_sim::Windows::new`. The
        // size is configured all the same, so what is missing is only the rectangle to centre it on.
        let game = Fake::attach("no-monitor", the_run(), |config| {
            config.screen = CONFIGURED;
            config.log_level = LogLevel::Verbose;
        });
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
