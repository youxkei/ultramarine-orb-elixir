//! **Where the window goes while orb is playing th06nc itself, and when it reaches the screen at all.**
//!
//! What `the_window_a_bot_plays_in.rs` asks of th06 asked of this game, and one thing more, because this
//! game's window is a different call and a different fight. th06's is decided inside `GameWindow::Create`
//! and stays where it was put; this one is asked for by a name the game looks up — `CreateWindowExW`, see
//! `Th06nc::hooks` — and **something moves it afterwards**. A launch whose `MoveWindow` was answered and
//! whose two `SetWindowPos` calls both carried `SWP_NOMOVE` was still measured in the middle of the
//! primary monitor, so `orb_core::window::keeps_its_corner` stopped asking who moves it and puts the
//! window back once a frame whoever did.
//!
//! **Which somebody sees.** A window that is on the screen from the moment it is made is one that flashes
//! in the middle of the monitor before the frame that puts it back — the whole reason the corner is
//! chosen being to keep a run nobody is watching out of the way of somebody working. So the window is
//! made without `WS_VISIBLE` and shown once its corner has held, which is what this file is about: the
//! creation, the frames of nothing on the screen, the show, and the put-back that starts the wait again.
//!
//! **And nothing of any of it for a launch somebody plays**: that window is the settings' own, visible
//! when it is made and never moved.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// `WS_VISIBLE`, written out the way `orb_core::window`'s own styles are: the one bit of the style that
/// says whether the window is on the screen the moment it is made.
const VISIBLE: u32 = 0x1000_0000;

/// `WS_EX_DLGMODALFRAME` and `WS_EX_TOOLWINDOW`, the frame that keeps such a window out of the way.
const DIALOG: u32 = 0x0000_0001;
const TOOL: u32 = 0x0000_0080;

/// And the style the game writes over orb's when it makes its own window over: `WS_EX_WINDOWEDGE` alone,
/// which is what th06c's window was measured holding — `exstyle 0x00000100`.
const THE_GAMES_OWN: u32 = 0x0000_0100;

/// **How long the corner has to hold before the window is shown**, as `orb_core::window` has it — in the
/// frames that function is asked on, which for this game is its own keyboard read.
///
/// Written out here rather than read from there, the way every other number an e2e test charges is: a
/// number shared between the code and the test agrees with itself whatever it says.
const THE_CORNER_HOLDS_FOR: u32 = 60;

/// How long a test waits for the show before calling it a window that is never shown. Twice the frames
/// above, because what the wait counts is the frames `keeps_its_corner` is asked on and this game's read
/// is asked rather more often than its frames come — see `game/th06nc/asm.md`, which has 66 reads to 60
/// frames measured over a stage.
const AND_NO_LONGER_THAN: u32 = 2 * THE_CORNER_HOLDS_FOR;

/// Where the game puts its own window, which for the real one is the middle of the primary monitor: the
/// two moves a launch of th06nc was measured making were to `1269,656` and `1749,926` on a 2560x1440
/// desktop, and the panel here is the declared one every other test over this game uses.
const THE_MIDDLE: (i32, i32) = (1280, 720);

/// The window the game asked the host for, of which there is one.
///
/// # Panics
/// Where the host was asked for none or for more than one — a second is one that flashed on the screen
/// before the one that stayed.
fn the_only_window(game: &Fake) -> orb_sim::Made {
    let made = game.sim().windows().made();
    assert_eq!(
        made.len(),
        1,
        "the host was asked for {} windows",
        made.len()
    );
    made[0]
}

/// A launch orb plays this game in, with its window made.
fn playing_it(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some("[]".to_owned());
            config.log_level = LogLevel::Verbose;
        },
    )
}

/// The frame the host was asked to show the window on, counted from the one after the window was made,
/// and `None` where it was not asked inside [`AND_NO_LONGER_THAN`] of them.
fn shown_on_the_frame(game: &Fake, window: orb_api::Hwnd) -> Option<u32> {
    for frame in 1..=AND_NO_LONGER_THAN {
        game.frame();
        if game.sim().windows().shown().contains(&window) {
            return Some(frame);
        }
    }
    None
}

#[test]
fn a_window_orb_plays_th06nc_in_is_made_hidden_in_the_corner() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window");
        let window = game.creates_its_window();
        let made = the_only_window(&game);

        assert_eq!(
            made.style & VISIBLE,
            0,
            "the window orb plays in was made on the screen:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.sim().windows().shown().is_empty(),
            "the window was shown before a frame had held its corner",
        );
        // The corner of the one monitor this launch is played on, frame counted in: what is asked for is
        // the whole window, so it is its own left and top that sit against the monitor's.
        assert_eq!(
            (made.asked.left, made.asked.top),
            (0, 0),
            "the window orb plays in is not in the monitor's top-left corner:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            made.ex_style & (DIALOG | TOOL),
            DIALOG | TOOL,
            "the window orb plays in was not framed as a dialog kept off the taskbar",
        );
        assert_eq!(
            made.handle, window,
            "the game was handed a window the host never made",
        );
    });
}

/// And the game's own show is swallowed while that corner is being settled — which is what the style
/// alone could not do: this game looks `ShowWindow` up and calls it, and a launch whose window was made
/// without `WS_VISIBLE` was on the screen a frame later anyway.
#[test]
fn the_games_own_show_of_it_is_swallowed_until_the_corner_has_held() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window-not-shown-by-the-game");
        let window = game.creates_its_window();

        assert_eq!(
            game.shows_its_window(),
            0,
            "orb answered the game's show as though the window had been up",
        );
        assert!(
            !game.sim().windows().is_visible(window),
            "the game's own show put the window orb plays in on the screen:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the place, which is the other way one call both moves a window and shows it: the middle of
        // the monitor is where the real one's window is measured being put.
        assert_ne!(
            game.places_its_window(THE_MIDDLE, true),
            0,
            "the host refused to place the window",
        );
        let placed = the_only_window(&game).asked;
        assert_eq!(
            (placed.left, placed.top),
            (0, 0),
            "the game placed its window in the middle and it stayed there:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            !game.sim().windows().is_visible(window),
            "the place the game asked to show its window with showed it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And it is still shown in the end, the asks having been answered rather than lost.
        let shown_on = shown_on_the_frame(&game, window);
        assert!(
            shown_on.is_some(),
            "the window nobody else showed was never shown by orb either:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.sim().windows().is_visible(window),
            "the window orb showed is not on the screen",
        );
    });
}

/// And once orb has shown it, the game's own show reaches the host again: what was being kept off the
/// screen was the window before its corner held, and nothing after.
#[test]
fn the_games_own_show_reaches_the_host_once_orb_has_shown_it() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window-shown-by-the-game");
        let window = game.creates_its_window();
        shown_on_the_frame(&game, window).unwrap_or_else(|| {
            panic!(
                "the window orb plays in was never shown:\n  {}",
                game.log().lines().join("\n  "),
            )
        });

        assert_ne!(
            game.shows_its_window(),
            0,
            "the game's own show was still being swallowed after orb had shown the window:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the dialog frame is put back where the game has written its own style over orb's**, which is
/// what the frame being kept every frame rather than on one is for: th06c's window was measured with an
/// extended style of `0x100` — no dialog frame and back in the taskbar — where orb had made it with both
/// bits, so the game had written over them some time after the window was made.
#[test]
fn the_dialog_frame_the_game_wrote_over_is_put_back() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window-framed-again");
        let window = game.creates_its_window();
        assert_eq!(
            the_only_window(&game).ex_style & (DIALOG | TOOL),
            DIALOG | TOOL,
            "the window orb plays in was not made framed as a dialog",
        );

        // The game makes its own window over, which is a style orb does not answer the writing of.
        assert!(
            game.sim().windows().set_ex_style(window, THE_GAMES_OWN),
            "the host had no window to write a style over",
        );
        game.frame();
        assert_eq!(
            the_only_window(&game).ex_style & (DIALOG | TOOL),
            DIALOG | TOOL,
            "the dialog frame the game wrote over was not put back:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And shown once it has held that corner — once, and not again on every frame after it.
#[test]
fn it_is_shown_once_its_corner_has_held() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window-shown");
        let window = game.creates_its_window();

        let shown_on = shown_on_the_frame(&game, window).unwrap_or_else(|| {
            panic!(
                "the window orb plays in was never shown:\n  {}",
                game.log().lines().join("\n  "),
            )
        });
        assert!(
            shown_on >= THE_CORNER_HOLDS_FOR,
            "the window was shown on frame {shown_on}, before its corner had held for \
             {THE_CORNER_HOLDS_FOR}:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.frames(THE_CORNER_HOLDS_FOR);
        assert_eq!(
            game.sim().windows().shown(),
            vec![window],
            "the window was shown again after it was already up",
        );
    });
}

/// And a window something moved is put back and made to hold its corner again before it is shown — which
/// is the whole point of the wait: the mover is the swapchain this game builds for the window it has just
/// made, and a window shown before that has happened is one that flashes in the middle of the monitor.
#[test]
fn a_window_something_moved_is_put_back_before_it_is_shown() {
    in_its_own_process(|| {
        let game = playing_it("th06nc-the-bots-window-moved");
        let window = game.creates_its_window();
        let asked = the_only_window(&game).asked;

        // The host moves it to the middle of the monitor, which is where the real one is measured.
        let moved_to = orb_api::Rect {
            left: 1280,
            top: 720,
            right: 1280 + asked.width(),
            bottom: 720 + asked.height(),
        };
        assert!(
            game.sim().windows().move_window(window, moved_to),
            "the host had no window to move",
        );
        game.frame();
        let put_back = the_only_window(&game).asked;
        assert_eq!(
            (put_back.left, put_back.top),
            (0, 0),
            "the window something moved was not put back in its corner:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.sim().windows().shown().is_empty(),
            "the window was shown on the frame it had to be put back on:\n  {}",
            game.log().lines().join("\n  "),
        );

        let shown_on = shown_on_the_frame(&game, window).unwrap_or_else(|| {
            panic!(
                "the window put back in its corner was never shown:\n  {}",
                game.log().lines().join("\n  "),
            )
        });
        assert!(
            shown_on >= THE_CORNER_HOLDS_FOR,
            "the window was shown on frame {shown_on} of holding its corner again, before it had held \
             it for {THE_CORNER_HOLDS_FOR}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And nothing of any of it where nobody asked orb to play: the window is the settings' own, on the
/// screen when it is made, where the game asked for it, and never put anywhere else.
#[test]
fn a_window_somebody_plays_th06nc_in_is_the_settings_own() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-not-the-bots-window",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
            },
        );
        let window = game.creates_its_window();
        let made = the_only_window(&game);

        assert_eq!(
            made.style & VISIBLE,
            VISIBLE,
            "a window somebody plays in was made hidden:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            made.ex_style & (DIALOG | TOOL),
            0,
            "a window somebody plays in was framed as a dialog",
        );
        let was_asked_for = made.asked;
        game.frames(AND_NO_LONGER_THAN);
        assert!(
            game.sim().windows().shown().is_empty(),
            "a window somebody plays in was shown by orb:\n  {}",
            game.log().lines().join("\n  "),
        );
        let now = the_only_window(&game).asked;
        assert_eq!(
            (now.left, now.top),
            (was_asked_for.left, was_asked_for.top),
            "a window somebody plays in was moved:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(window.0, 0, "the host made no window");
    });
}
