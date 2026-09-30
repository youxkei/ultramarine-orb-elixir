//! **The window orb's rewrite of `CreateWindowExA` hands straight on, over a launch of th10.**
//!
//! What `the_windows_orb_leaves_alone_of_th08.rs` asks of that game, asked of this one. `the_window_of_th10.rs`
//! is every window orb *decides*; this is the other side of the same rewrite — the ask it must not touch:
//!
//! - **a monitor that cannot be read.** Fullscreen is the monitor's rectangle and a size is centred on it, so a
//!   host that will not say where its panel is is a host with nothing to lay a window out against. The window
//!   is then the one the game asked for, which is the window every launch before orb had.
//!
//! **th08's other case, a window that is not the game's own class, is not asked here**: th10 never makes one.
//! Both of its calls to the imported `CreateWindowExA` (0x4397dc, 0x43982c) name `BASE` at `0x46f698`, and the
//! one other window it puts up, the startup dialog (`DialogBoxParamA`, 0x438b8c), is made inside `USER32`
//! rather than through the game's import orb rewrites.
//!
//! What is read back is what the host was really asked for, out of `orb_sim::Windows::made`.

use crate::fake::th10::{ASKED_FOR_FULL_SCREEN, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};
use orb_sim::Made;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The size the launch below is configured for, which is what makes the assertions say something: a window
/// orb laid out would be this size and centred, and this one is the game's own ask instead.
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

/// On a host that will not say where its panel is, the game's own window is what the game gets.
#[test]
fn a_monitor_that_cannot_be_read_leaves_th10s_window_as_it_asked_for_it() {
    in_its_own_process(|| {
        let game = Fake::attach_without_a_monitor(
            Display::agreed(120),
            "th10-no-monitor",
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
        assert!(
            !game.image().is_windowed(),
            "the launch's `th10.cfg` asks for a window, so the ask below is not the one the game makes",
        );

        game.creates_its_window();
        let made = made(&game);
        assert_eq!(
            (made.asked.left, made.asked.top),
            ASKED_FOR_FULL_SCREEN.at,
            "the position the game asked for was rewritten",
        );
        assert_eq!(
            (made.asked.width(), made.asked.height()),
            ASKED_FOR_FULL_SCREEN.size,
            "the size the game asked for was rewritten",
        );
        assert_eq!(
            (made.style, made.ex_style),
            (ASKED_FOR_FULL_SCREEN.style, 0),
            "the style the game asked for was rewritten",
        );
        assert!(
            !game.log().said(" — window at "),
            "orb laid a window out against a monitor it could not read:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
