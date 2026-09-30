//! **The window orb makes for th07: the size asked for, the monitor's real pixels, and the black beside the
//! game.**
//!
//! The arithmetic is orb's and `the_window.rs` holds it over th06; what this game brings is the two ends of it.
//! **The ask** is th07's own `CreateWindowExA`, rewritten inside the call — `Fake::creates_its_window` passes
//! the game's own class and the size `Th07::content_size` names, and every argument of it is orb's by the time
//! the host sees one. **And the setting underneath**, which this game keeps in its own configuration: a launch
//! that left the display taken exclusively would have a window orb cannot resize, so the flag is written before
//! the game reads it, inside the one call everything about its window is decided in — `Th07::hooks`'
//! `create_window`, at 0x434a80.
//!
//! What every rectangle below is worked out from is the host these launches declare: a scaled monitor that reads
//! one size to a process which has not asked otherwise and another once it has, with a frame round a window of a
//! chosen size. Both belong to the host and neither is a number a test could otherwise move.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_api::Rect;
use orb_config::{LogLevel, Screen};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The window in the settings for the e2e tests about a chosen size, and the whole window this host needs to give
/// it that client: 1280x720 plus the 6x40 frame.
const CLIENT: (u32, u32) = (1280, 720);
const WHOLE: (i32, i32) = (1286, 760);

/// What the declared monitor reads as before and after the process says it is DPI aware.
const SCALED: (i32, i32) = (2560, 1440);
const REAL: (i32, i32) = (3840, 2160);

/// A launch with that window in `orb.yaml`, with the game's window made and then its device, as a real launch
/// makes them — the device being where orb redirects the `Present` the black beside the game is written around.
///
/// Nothing of a run is played: what a window is does not depend on what is on screen, and the game sits where a
/// launch leaves it.
fn launched(name: &str, screen: Screen) -> Box<Fake> {
    let game = Fake::attach_before_its_device(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        move |config| {
            config.screen = screen;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.creates_its_window();
    game.finds_its_device();
    game
}

/// The one window the host was asked for, and the client it came out with.
///
/// # Panics
/// Where the host was asked for none, or for more than one — a second window is a window that flashed on the
/// screen before the one that stayed, which is the thing the rewrite exists to avoid.
fn the_window(game: &Fake) -> orb_sim::Made {
    let made = game.sim().windows().made();
    assert_eq!(
        made.len(),
        1,
        "the host was asked for {} windows, and one of them was on the screen before the other",
        made.len(),
    );
    made[0]
}

/// A game configured for full screen is put in a window before it makes one, once.
///
/// **Every answer below needs a window to give it.** A game that has taken the display exclusively has none to
/// resize, and by the time anything of orb's runs per frame the device already exists — so the setting has to be
/// written before the game reads it, which is inside the one call everything about its window is decided in.
///
/// Once, and only where the game was going to take the display: the flag is spent on the first window a launch
/// makes, and a game already configured for a window is left as it is.
#[test]
fn a_th07_configured_for_full_screen_is_put_in_a_window_before_it_makes_one() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-window-overruled",
            Work::wandering(WORK_US),
            |config| {
                config.screen = Screen::Fullscreen;
                config.log_level = LogLevel::Verbose;
            },
        );
        game.image().is_configured_for_a_window(false);
        assert!(
            !game.image().asks_for_a_window(),
            "the game was already configured for a window, so there is nothing here to overrule",
        );
        game.creates_its_window();
        assert!(
            game.image().asks_for_a_window(),
            "the game made its window with the display still taken exclusively, which is a window \
             orb cannot resize:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.log()
                .said("borderless: overrode the game's fullscreen setting"),
            "orb did not say it had overruled the setting:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the flag is spent: a second window is made under whatever setting the game has by then, there
        // being nothing left of a launch's one chance to overrule it.
        game.image().is_configured_for_a_window(false);
        game.creates_its_window();
        assert!(
            !game.image().asks_for_a_window(),
            "the setting was overruled again for a window that is not the launch's first",
        );
    });
}

/// And a game already configured for a window is left alone, which is the other side of the same read.
#[test]
fn a_th07_already_in_a_window_is_left_as_it_is() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-window-left-alone",
            Work::wandering(WORK_US),
            |config| {
                config.screen = Screen::Fullscreen;
                config.log_level = LogLevel::Verbose;
            },
        );
        game.image().is_configured_for_a_window(true);
        game.creates_its_window();
        assert!(
            game.image().asks_for_a_window(),
            "the setting orb writes is not the one it read",
        );
        assert!(
            !game
                .log()
                .said("borderless: overrode the game's fullscreen setting"),
            "orb overruled a setting that already said what it wanted:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// The client is exactly the size asked for, and whatever frame the host adds is outside it.
///
/// A window is asked for by its client and the frame goes round it: what `orb.yaml` names is how much game there
/// is, and the window is bigger than that by however much this host's frame costs.
#[test]
fn the_client_of_th07s_window_is_the_size_asked_for_and_the_frame_is_outside_it() {
    in_its_own_process(|| {
        let game = launched(
            "th07-the-window-sized",
            Screen::Window {
                width: CLIENT.0,
                height: CLIENT.1,
            },
        );
        let made = the_window(&game);

        // The whole window carries the frame, so it is the larger of the two numbers…
        assert_eq!(
            (made.asked.width(), made.asked.height()),
            WHOLE,
            "the window asked for is not the client plus this host's 6x40 frame",
        );
        // …and the client inside it is the size in the settings, to the pixel.
        assert_eq!(
            (made.client.width(), made.client.height()),
            (CLIENT.0 as i32, CLIENT.1 as i32),
            "the client is not the size the settings asked for",
        );
        // And centred on the monitor's real pixels: (3840−1286)/2 = 1277 and (2160−760)/2 = 700.
        assert_eq!(
            (made.asked.left, made.asked.top),
            ((REAL.0 - WHOLE.0) / 2, (REAL.1 - WHOLE.1) / 2),
        );
        assert_eq!((made.asked.left, made.asked.top), (1277, 700));
        assert!(
            game.log()
                .said("screen: 1280x720 — window at 1277,700 sized 1286x760, client 1280x720"),
            "the log does not say what window came out:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Display scaling is ignored, which is what makes every size the monitor's real pixels.
///
/// What is held is which *side* of `SetProcessDPIAware` each read fell on, which is the whole of it: the answer
/// being the real pixels follows, and a read on the wrong side would answer the scaled ones however the
/// arithmetic after it went.
#[test]
fn the_monitor_th07_is_laid_out_on_is_its_real_pixels_once_the_process_says_it_is_dpi_aware() {
    in_its_own_process(|| {
        let game = launched("th07-the-window-dpi", Screen::Fullscreen);

        let reads = game.sim().windows().monitor_reads();
        assert!(!reads.is_empty(), "orb never read the monitor at all");
        assert!(
            reads.iter().all(|(aware, _)| *aware),
            "orb read the monitor before saying it was DPI aware: {reads:?}",
        );
        assert!(
            reads
                .iter()
                .all(|(_, rect)| (rect.width(), rect.height()) == REAL),
            "a read did not answer the monitor's real pixels: {reads:?}",
        );
        // Which is a different size from the one the same monitor answers before the call, and that is what
        // makes the ordering worth asserting rather than a formality.
        assert_ne!(SCALED, REAL);
        assert!(
            game.log()
                .said("screen: display scaling is being ignored, sizes are real pixels"),
            "the log does not say scaling was turned off:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Borderless fullscreen keeps th07's own ratio and blacks the rest, with no frame to remove.
///
/// The window is the monitor, at its origin and its whole size, with the game letterboxed inside it — 4:3 in
/// 16:9, which is what `Th07::content_size` being 640x480 comes to.
#[test]
fn borderless_fullscreen_fills_the_monitor_with_th07_letterboxed_inside_it() {
    in_its_own_process(|| {
        let game = launched("th07-the-window-fullscreen", Screen::Fullscreen);
        let made = the_window(&game);

        // Both ways: the window covers the monitor and so does the client, which is the borderless part — a
        // window with a frame would have one of the two smaller than the other.
        assert_eq!((made.asked.left, made.asked.top), (0, 0));
        assert_eq!((made.asked.width(), made.asked.height()), REAL);
        assert_eq!((made.client.width(), made.client.height()), REAL);
        assert!(
            !made.framed,
            "a borderless window was asked for with a style that has a frame",
        );
        assert!(
            game.log()
                .said("screen: fullscreen — window at 0,0 sized 3840x2160, client 3840x2160"),
            "the log does not say the window covered the monitor:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The game fills it top to bottom on a 16:9 monitor, so the black is down the sides: 4:3 inside
        // 3840x2160 is 2880 wide, 480 either side.
        let game_at =
            unsafe { orb_core::window::letterbox() }.expect("a letterbox for a window that exists");
        assert_eq!(
            (game_at.width(), game_at.height()),
            (2880, 2160),
            "the game is not its own ratio inside the monitor",
        );
        assert_eq!(game_at.left, 480);
        assert_eq!(REAL.0 - game_at.right, 480);
    });
}

/// And a client of the game's own ratio leaves no black at all, which orb says rather than writing over the
/// game.
#[test]
fn a_four_three_client_leaves_th07_no_black_and_orb_says_so() {
    in_its_own_process(|| {
        let game = launched(
            "th07-the-window-no-black",
            Screen::Window {
                width: 1600,
                height: 1200,
            },
        );
        let made = the_window(&game);
        assert_eq!((made.client.width(), made.client.height()), (1600, 1200));

        let game_at =
            unsafe { orb_core::window::letterbox() }.expect("a letterbox for a window that exists");
        assert_eq!(game_at, Rect::sized(1600, 1200), "the game left black over");

        unsafe { orb_core::window::write_beside(&[String::from("CH 05")]) };
        assert!(
            game.log()
                .said("screen: client 1600x1200, game 1600x1200 at 0,0 — no black to write in"),
            "orb did not say there was nowhere to write:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
