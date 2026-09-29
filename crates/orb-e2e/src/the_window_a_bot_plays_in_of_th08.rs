//! **Where th08's window goes while orb is playing the game itself**: small, in the top-left corner of the
//! leftmost monitor, and framed as a dialog.
//!
//! What `the_window_a_bot_plays_in_of_th07.rs` asks of that game, asked of this one — and th07's rather than
//! th06nc's, because this game makes its window the way th07 does: one `CreateWindowExA` on the class `BASE`,
//! imported, and the window left where it was made — `Th08::hooks` declares none of the lookups th06nc moves
//! its window through. So what `--autoplay` decides is decided inside that one call.
//!
//! **The dialog frame and the corner are the same instruction**: keep out of the way. The frame is what
//! `WS_EX_DLGMODALFRAME` makes of it, and `WS_EX_TOOLWINDOW` beside it is what keeps it off the taskbar and
//! out of alt-tab.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The monitor this is played on, as the laid-out host declares it in real pixels — the same one
/// `the_window_of_th08` measures against.
const MONITOR: (i32, i32) = (3840, 2160);
/// And the client the bot's window is given inside its frame: half of th08's own 640x480.
const BOT_CLIENT: (i32, i32) = (320, 240);

/// `WS_EX_DLGMODALFRAME` and `WS_EX_TOOLWINDOW`, written out the way `orb_core::window`'s own styles are.
const DIALOG: u32 = 0x0000_0001;
const TOOL: u32 = 0x0000_0080;

/// The one window the host was asked for.
///
/// # Panics
/// Where it was asked for none or for more than one — a second is one that flashed on the screen before the
/// one that stayed.
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
fn a_bots_window_over_th08_is_small_in_the_corner_and_framed_as_a_dialog() {
    in_its_own_process(|| {
        // Fullscreen asked for in the settings, which is what says the bot's window is not the settings':
        // a window this test finds in a corner is one `--autoplay` put there.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-bots-window",
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
        assert_eq!(
            (made.asked.left, made.asked.top),
            (0, 0),
            "the bot's window is not in the monitor's top-left corner",
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
fn a_window_somebody_plays_th08_in_is_the_settings_own() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-not-the-bots-window",
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
