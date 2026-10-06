//! **Where th11's window goes while orb is playing the game itself**: small, in the top-left corner of the
//! leftmost monitor, and framed as a dialog.
//!
//! What `the_window_a_bot_plays_in_of_th10.rs` asks of that game, asked of this one — and for th10's reason, this
//! game making its window the way th10 does: 0x446ae0 registers the class `BASE` (0x446b5a) and makes one window on
//! it through the imported `CreateWindowExA` (0x446c28 or 0x446cb5, whichever `th11.cfg` says), and the window is
//! left where it was made — `Th11::hooks` declares none of the lookups th06nc moves its window through. So what
//! `--autoplay` decides is decided inside that one call.
//!
//! **The dialog frame and the corner are the same instruction**: keep out of the way. The frame is what
//! `WS_EX_DLGMODALFRAME` makes of it, and `WS_EX_TOOLWINDOW` beside it is what keeps it off the taskbar and
//! out of alt-tab.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The monitor this is played on, as the laid-out host declares it in real pixels — the same one
/// `the_window_of_th11` measures against.
const MONITOR: (i32, i32) = (3840, 2160);
/// And the client the bot's window is given inside its frame: half of th11's own 640x480.
const BOT_CLIENT: (i32, i32) = (320, 240);

/// `WS_EX_DLGMODALFRAME`, `WS_EX_TOOLWINDOW` and `WS_EX_NOACTIVATE`, written out the way `orb_core::window`'s own
/// styles are.
const DIALOG: u32 = 0x0000_0001;
const TOOL: u32 = 0x0000_0080;
const NO_ACTIVATE: u32 = 0x0800_0000;
/// `WS_VISIBLE`, which says whether a window is on the screen the moment it is made.
const VISIBLE: u32 = 0x1000_0000;
/// `D3DCREATE_HARDWARE_VERTEXPROCESSING`, the flag th11 first makes its device with (0x446f5d), and
/// `D3DCREATE_NOWINDOWCHANGES`.
const HARDWARE_VERTEX_PROCESSING: u32 = 0x40;
const NO_WINDOW_CHANGES: u32 = 0x800;

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
fn a_bots_window_over_th11_is_small_in_the_corner_and_framed_as_a_dialog() {
    in_its_own_process(|| {
        // Fullscreen asked for in the settings, which is what says the bot's window is not the settings':
        // a window this test finds in a corner is one `--autoplay` put there.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-bots-window",
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
        assert_eq!(
            made.ex_style & NO_ACTIVATE,
            NO_ACTIVATE,
            "the bot's window can be activated, taking the focus off whatever somebody is working in",
        );
    });
}

/// **And the window the bot plays in is not brought to the front when the game asks for it**, which th11's WinMain
/// does once its window is made (0x4457d8) — the focus left on whatever somebody is working in, `WS_EX_NOACTIVATE`
/// keeping the window from a click and from the system and not from the game's own `SetForegroundWindow`.
#[test]
fn the_window_a_bot_plays_th11_in_is_not_brought_forward_by_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-bots-window-not-forward",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let window = game.creates_its_window();
        game.brings_its_window_forward(window);
        assert_eq!(
            game.sim().windows().brought_forward(),
            Vec::new(),
            "the game's own SetForegroundWindow brought the bot's window to the front",
        );
    });
}

/// **And the window the bot plays in is made hidden and shown by orb**, without being activated: made visible, a
/// launch's window took the focus off whatever somebody was working in the moment `CreateWindowExA` made it, with
/// `WS_EX_NOACTIVATE` on it and the game's own `SetForegroundWindow` swallowed — a new process's first visible
/// window being one Windows brings to the front.
#[test]
fn the_window_a_bot_plays_th11_in_is_made_hidden_and_shown_by_orb() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-bots-window-made-hidden",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let window = game.creates_its_window();
        let made = the_only_window(&game);
        assert_eq!(
            made.style & VISIBLE,
            0,
            "the bot's window was made visible, which is the window shown and activated as it is made",
        );
        assert_eq!(
            game.sim().windows().shown(),
            vec![window],
            "the bot's window made hidden was not shown",
        );
    });
}

/// **And the device the bot's window is drawn through made with Direct3D told to leave that window alone** —
/// `D3DCREATE_NOWINDOWCHANGES` beside the game's own `D3DCREATE_HARDWARE_VERTEXPROCESSING`, there being no
/// `D3DCREATE_NOWINDOWCHANGES` in the game's own flags — and made with the game's own flags alone where somebody
/// plays.
#[test]
fn the_device_of_th11_is_made_to_leave_the_window_a_bot_plays_in_alone() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-bots-device",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let window = game.creates_its_window();
        game.makes_its_direct3d_and_its_device(window);
        assert_eq!(
            game.the_device_was_made_with(),
            Some(HARDWARE_VERTEX_PROCESSING | NO_WINDOW_CHANGES),
            "the device of the bot's window was not made to leave the window alone",
        );
    });
}

#[test]
fn the_device_of_th11_somebody_plays_is_made_with_the_games_own_flags() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-not-the-bots-device",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let window = game.creates_its_window();
        game.makes_its_direct3d_and_its_device(window);
        assert_eq!(
            game.the_device_was_made_with(),
            Some(HARDWARE_VERTEX_PROCESSING),
            "the device of a window somebody plays in was not made with the game's own flags",
        );
    });
}

/// **And the focus the bot's window takes all the same handed back to the window that had it**, whatever brought it
/// forward — a game whose device or whose own code orb has no answer for: the window in front when orb attached is
/// in front again a frame after the bot's came to the front.
#[test]
fn the_focus_the_window_a_bot_plays_th11_in_takes_is_handed_back() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-bots-window-gives-the-focus-back",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let somebody_elses = game.sim().display().foreground();
        let window = game.creates_its_window();
        game.sim().display().set_foreground(window);
        game.frames(2);
        assert_eq!(
            game.sim().display().foreground(),
            somebody_elses,
            "the focus the bot's window took was not handed back:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the window somebody plays in is brought forward as the game asks**, that being their window.
#[test]
fn the_window_somebody_plays_th11_in_is_brought_forward_by_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-not-the-bots-window-forward",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let window = game.creates_its_window();
        game.brings_its_window_forward(window);
        assert_eq!(
            game.sim().windows().brought_forward(),
            vec![window],
            "the game's own SetForegroundWindow did not reach the host for a window somebody plays in",
        );
    });
}

/// And nothing of that where nobody asked orb to play: a launch that is somebody's own gets the window the
/// settings ask for.
#[test]
fn a_window_somebody_plays_th11_in_is_the_settings_own() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-not-the-bots-window",
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
            made.ex_style & (DIALOG | TOOL | NO_ACTIVATE),
            0,
            "a window somebody plays in was framed as a dialog or kept from being activated",
        );
    });
}
