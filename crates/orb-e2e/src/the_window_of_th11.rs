//! **The window orb makes for th11: the size asked for, the monitor's real pixels, and a borderless window over
//! the whole of it.**
//!
//! What `the_window_of_th10.rs` asks of that game, asked of this one: the arithmetic is orb's and
//! `the_window.rs` holds it over th06, and what this game brings is the ask — its own `CreateWindowExA` at
//! 0x446c28 or 0x446cb5, rewritten inside the call, on the class `BASE` and with the size `Th11::content_size`
//! names, the 640x480 back buffer.
//!
//! **What th08's file asks and this one does not**, each for what th11's exe and orb's code do:
//!
//! - **the full-screen setting overruled, and a window left alone.** th08's is written inside
//!   `GameWindow::CreateGameWindow`, `Th08::hooks`' `create_window`; th11's byte (`CFG_WINDOWED`, the
//!   configuration's window size at `0x4c3465`, read at 0x446b60) is read inside 0x446ae0, which is entered with
//!   the instance in `ebx` (0x445733) and which `Th11::hooks` declares no `create_window` for — so
//!   `Th11::force_windowed` is never reached, and a th11 set to full screen takes the display as `th11.cfg` says.
//! - **the game letterboxed inside the window, and the black beside it.** orb's letterbox is a redirect of
//!   `IDirect3DDevice8::Present`'s slot made at the device's setup, `Hooks::init_device`, and th11 declares
//!   none and presents through `IDirect3DDevice9` (0x4465e9, 0x446395, 0x44680c) — so the back buffer is
//!   stretched over the whole client, and a rectangle `orb_core::window::letterbox` works out is one nothing
//!   presents into.
//!
//! What every rectangle below is worked out from is the host these launches declare: a scaled monitor that
//! reads one size to a process which has not asked otherwise and another once it has, with a frame round a
//! window of a chosen size. Both belong to the host and neither is a number a test could otherwise move.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{LogLevel, Screen};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The window in the settings for the e2e tests about a chosen size, and the whole window this host needs to
/// give it that client: 1280x720 plus the 6x40 frame.
const CLIENT: (u32, u32) = (1280, 720);
const WHOLE: (i32, i32) = (1286, 760);

/// What the declared monitor reads as before and after the process says it is DPI aware.
const SCALED: (i32, i32) = (2560, 1440);
const REAL: (i32, i32) = (3840, 2160);

/// A launch with that window in `orb.yaml`, with the game's window made.
///
/// Nothing of a run is played: what a window is does not depend on what is on screen, and the game sits where
/// a launch leaves it.
fn launched(name: &str, screen: Screen) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        move |config| {
            config.screen = screen;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.creates_its_window();
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

/// The client is exactly the size asked for, and whatever frame the host adds is outside it.
///
/// A window is asked for by its client and the frame goes round it: what `orb.yaml` names is how much game
/// there is, and the window is bigger than that by however much this host's frame costs.
#[test]
fn the_client_of_th11s_window_is_the_size_asked_for_and_the_frame_is_outside_it() {
    in_its_own_process(|| {
        let game = launched(
            "th11-the-window-sized",
            Screen::Window {
                width: CLIENT.0,
                height: CLIENT.1,
            },
        );
        let made = the_window(&game);

        assert_eq!(
            (made.asked.width(), made.asked.height()),
            WHOLE,
            "the window asked for is not the client plus this host's 6x40 frame",
        );
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
/// What is held is which *side* of `SetProcessDPIAware` each read fell on, which is the whole of it: the
/// answer being the real pixels follows, and a read on the wrong side would answer the scaled ones however the
/// arithmetic after it went.
#[test]
fn the_monitor_th11_is_laid_out_on_is_its_real_pixels_once_the_process_says_it_is_dpi_aware() {
    in_its_own_process(|| {
        let game = launched("th11-the-window-dpi", Screen::Fullscreen);

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
        assert_ne!(SCALED, REAL);
        assert!(
            game.log()
                .said("screen: display scaling is being ignored, sizes are real pixels"),
            "the log does not say scaling was turned off:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// **Nothing is painted over th11's picture where a letterbox's black would be**: its back buffer is stretched
/// over the whole client, so the bars beside a 4:3 picture on this 16:9 monitor are the picture's own edges, and
/// orb's numbers written there through the GDI are over the game — covered by its next present, and back on the
/// one after.
#[test]
fn nothing_is_painted_over_th11s_stretched_picture() {
    in_its_own_process(|| {
        let game = launched("th11-the-window-nothing-beside", Screen::Fullscreen);
        game.frames(600);
        let painted: Vec<_> = game
            .sim()
            .windows()
            .written()
            .into_iter()
            .filter(|written| !written.lines.is_empty())
            .collect();
        assert!(
            painted.is_empty(),
            "orb painted {} stack(s) of lines over th11's picture, the first {:?}",
            painted.len(),
            painted.first().map(|written| &written.lines),
        );
    });
}

/// Borderless fullscreen is the monitor, at its origin and its whole size, with no frame to remove.
#[test]
fn borderless_fullscreen_gives_th11_a_window_over_the_whole_monitor() {
    in_its_own_process(|| {
        let game = launched("th11-the-window-fullscreen", Screen::Fullscreen);
        let made = the_window(&game);

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
    });
}
