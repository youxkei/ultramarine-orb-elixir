//! **Where the window goes while orb is playing the game itself**: small, in the monitor's bottom-left
//! corner, and framed as a dialog.
//!
//! A run orb plays is a run nobody is watching, and it is played on the machine somebody is working on. The
//! settings say where a window a player asked for goes — fullscreen, or a size of their own — and none of that
//! is what a bot's window is for, so `--autoplay` decides this one itself whatever the file says.
//!
//! **The dialog frame and the corner are the same instruction**: keep out of the way. The frame is what
//! `WS_EX_DLGMODALFRAME` makes of it, and `WS_EX_TOOLWINDOW` beside it is what keeps it off the taskbar and
//! out of alt-tab — which is the half the *reason* picks rather than the word, and the half to take out if a
//! window nobody can alt-tab to turns out to be worse than one in the way.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The monitor this is played on, as the laid-out host declares it in real pixels — the same one
/// `the_window_of_th07` measures against.
const MONITOR: (i32, i32) = (3840, 2160);
/// And the client the bot's window is given inside its frame: half of 妖々夢's own 640x480, which is small
/// enough to sit in a corner and big enough to see what the bot is doing to the run.
const BOT_CLIENT: (i32, i32) = (320, 240);

/// `WS_EX_DLGMODALFRAME` and `WS_EX_TOOLWINDOW`, written out the way `orb_core::window`'s own styles are.
const DIALOG: u32 = 0x0000_0001;
const TOOL: u32 = 0x0000_0080;

/// The one window the host was asked for.
///
/// # Panics
/// Where it was asked for none or for more than one — a second is one that flashed on the screen before the
/// one that stayed, which is what deciding the size inside the game's own creating call exists to avoid.
fn the_only_window(game: &Fake) -> orb_sim::Made {
    let made = game.sim().windows().made();
    assert_eq!(
        made.len(),
        1,
        "the host was asked for {} windows",
        made.len(),
    );
    made[0]
}

#[test]
fn a_bots_window_is_small_in_the_corner_and_framed_as_a_dialog() {
    in_its_own_process(|| {
        // Fullscreen asked for in the settings, which is what says the bot's window is not the settings':
        // a window this test finds in a corner is one `--autoplay` put there.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-bots-window",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.screen = Screen::Fullscreen;
                config.log_level = LogLevel::Verbose;
            },
        );
        game.creates_its_window();
        let made = the_only_window(&game);

        assert_eq!(
            (made.client.width(), made.client.height()),
            BOT_CLIENT,
            "the bot was given a window the size the settings asked for",
        );
        // The corner, and the frame counted in: what is asked for is the whole window, so it is its own
        // left and bottom that sit against the monitor's.
        assert_eq!(
            (made.asked.left, made.asked.bottom),
            (0, MONITOR.1),
            "the bot's window is not in the monitor's bottom-left corner",
        );
        assert_eq!(
            made.ex_style & (DIALOG | TOOL),
            DIALOG | TOOL,
            "the bot's window was not framed as a dialog kept off the taskbar",
        );
    });
}

/// And nothing of that where nobody asked orb to play: a launch that is somebody's own gets the window the
/// settings ask for.
#[test]
fn a_window_somebody_plays_in_is_the_settings_own() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-not-the-bots-window",
            Work::wandering(WORK_US),
            |config| {
                config.screen = Screen::Fullscreen;
                config.log_level = LogLevel::Verbose;
            },
        );
        game.creates_its_window();
        let made = the_only_window(&game);

        assert_eq!(
            (made.asked.width(), made.asked.height()),
            MONITOR,
            "a launch nobody asked to play itself did not get the fullscreen it asked for",
        );
        assert_eq!(
            made.ex_style & (DIALOG | TOOL),
            0,
            "a window somebody plays in was framed as a dialog",
        );
    });
}
