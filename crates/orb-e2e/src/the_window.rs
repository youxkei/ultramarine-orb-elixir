//! **The window orb makes: the size asked for, the monitor's real pixels, and the black beside the game.**
//!
//! What each e2e test holds is the layout orb has to come out with on the host it declares — a scaled
//! monitor that reads one size to a process which has not asked otherwise and another once it has, with a
//! frame round a window of a chosen size. Both of those belong to the host and neither is a number a test
//! could otherwise move, which is why they are declared through [`Panel::scaled`]: `orb_sim::Monitor`
//! answers two sizes for one panel and `orb_sim::Windows` charges the frame on the way in and gives it
//! back on the way out. The values are that declaration's and stand for any host of the same shape.
//!
//! **The game asks for a window and orb rewrites every argument of the ask.** `Fake::creates_its_window`
//! is that call — 646x505 at 17,23 with a caption, which is nothing like any answer below — so what these
//! read back is the window the host was really asked for, out of `orb_sim::Windows::made`. The arithmetic
//! underneath has its own tests — `window.rs`'s `a_window_is_centred_on_its_monitor` and the four beside
//! it — and what these add is the part arithmetic cannot reach: which side of `SetProcessDPIAware` the
//! monitor was read on, and the client a window of the size asked for really comes out with.

use crate::fake::{Launched, Panel, in_its_own_process, th06::Fake, th06::the_run};
use orb_api::{Bar, Rect};
use orb_config::Screen;
use orb_sim::{Metric, Written};

/// How far the text keeps from the edges of the window, and how tall a line of it is — the two numbers
/// every rectangle below is worked out from. Written out rather than read off `orb_core::window`, which keeps
/// both to itself: what an e2e test holds is the geometry that came out, and a test that took its expected
/// numbers from the code under test would agree with it whatever either said.
const BAR_MARGIN: i32 = 8;
const BAR_TEXT_HEIGHT: i32 = 30;

/// The window in the settings for the e2e tests about a chosen size, and the whole window this host
/// needs to give it that client: 1280x720 plus the 6x40 frame.
const CLIENT: (u32, u32) = (1280, 720);
const WHOLE: (i32, i32) = (1286, 760);

/// What the declared panel reads as before and after the process says it is DPI aware.
const SCALED: (i32, i32) = (2560, 1440);
const REAL: (i32, i32) = (3840, 2160);

/// A game configured for full screen is put in a window before it makes one, once.
///
/// **Every answer below needs a window to give it.** A game that has taken the display exclusively has
/// none to resize, and by the time anything of orb's runs per frame the device already exists — so the
/// setting has to be written before the game reads it, which is inside the one call everything about its
/// window is decided in: `GameWindow::Create`. That is also why nothing flashes on the screen.
///
/// Once, and only where the game was going to take the display: the flag is spent on the first window a
/// launch makes, and a game already configured for a window is left as it is.
#[test]
fn a_game_configured_for_full_screen_is_put_in_a_window_before_it_makes_one() {
    in_its_own_process(|| {
        let game = Fake::attach_to_a_panel(
            Panel::scaled(),
            "the-window-overruled",
            the_run(),
            |config| config.screen = Screen::Fullscreen,
        );
        assert!(
            !game.image().windowed(),
            "the game was already configured for a window, so there is nothing here to overrule",
        );
        game.creates_its_window();
        assert!(
            game.image().windowed(),
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

        // And the flag is spent: a second window is made under whatever setting the game has by then,
        // there being nothing left of a launch's one chance to overrule it.
        game.image().set_windowed(false);
        game.creates_its_window();
        assert!(
            !game.image().windowed(),
            "the setting was overruled again for a window that is not the launch's first",
        );
    });
}

/// And a game already configured for a window is left alone, which is the other side of the same read.
#[test]
fn a_game_already_in_a_window_is_left_as_it_is() {
    in_its_own_process(|| {
        let game = Fake::attach_to_a_panel(
            Panel::scaled(),
            "the-window-left-alone",
            the_run(),
            |config| config.screen = Screen::Fullscreen,
        );
        game.image().set_windowed(true);
        game.creates_its_window();
        assert!(
            game.image().windowed(),
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

/// A launch with that window in `orb.yaml`, with the game's window made.
///
/// Nothing of a run is played: what a window is does not depend on what is on screen, and the game sits
/// where a launch leaves it.
fn launched(name: &str, screen: Screen) -> Box<Fake> {
    let game = Fake::attach_to_a_panel(Panel::scaled(), name, the_run(), move |config| {
        config.screen = screen;
    });
    game.creates_its_window();
    game
}

/// The one window the host was asked for, and the client it came out with.
///
/// # Panics
/// Where the host was asked for none, or for more than one — a second window is a window that flashed
/// on the screen before the one that stayed, which is the thing the rewrite exists to avoid.
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

/// The stack of lines this host was last asked to write in the black beside the game.
///
/// The last rather than the only one: a launch writes one whenever the lines change, and what an e2e test
/// asserts about is the one its own call put there.
///
/// # Panics
/// Where the host has been asked for none, which is orb finding nowhere to write.
fn written(game: &Fake) -> Written {
    game.sim()
        .windows()
        .written()
        .pop()
        .expect("a stack of lines written beside the game")
}

/// The client is exactly the size asked for, and the frame this machine adds is outside it.
///
/// Measured: `screen: 1280x720` came out as `screen: 1280x720 — window at 1277,700 sized 1286x760,
/// client 1280x720`. The frame is the **6x40** between the two, and the window is centred on a monitor
/// read as 3840x2160. Still `client 1280x720` when the device was created.
#[test]
fn the_client_is_the_size_asked_for_and_the_frame_is_outside_it() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-sized",
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
            "the window asked for is not the client plus this host's 6x40 frame"
        );
        // …and the client inside it is the size in the settings, to the pixel. Which is the claim: what
        // `orb.yaml` says is how much game there is, whatever this machine's frames cost.
        assert_eq!(
            (made.client.width(), made.client.height()),
            (CLIENT.0 as i32, CLIENT.1 as i32),
            "the client is not the size the settings asked for"
        );
        // And centred on the monitor's real pixels: (3840−1286)/2 = 1277 and (2160−760)/2 = 700, which
        // is what the log line above read.
        assert_eq!(
            (made.asked.left, made.asked.top),
            ((REAL.0 - WHOLE.0) / 2, (REAL.1 - WHOLE.1) / 2)
        );
        assert_eq!((made.asked.left, made.asked.top), (1277, 700));

        // Said in the log the same way, since that line is what somebody reading a real run has: the
        // window that was asked for and the client that came of it, which are two different numbers and
        // the reason the line carries both.
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
/// Measured on the same monitor: it reads as **2560x1440** before `SetProcessDPIAware` and **3840x2160**
/// after. Without the call every size would have been scaled behind the game's back — a 1280x720 client
/// asked for on a 3840x2160 panel would have been laid out against 2560x1440.
///
/// What is held is which *side* of the call each read fell on, which is the whole of it: the answer being
/// the real pixels follows, and a read on the wrong side would answer the scaled ones however the
/// arithmetic after it went.
#[test]
fn the_monitor_is_its_real_pixels_once_the_process_says_it_is_dpi_aware() {
    in_its_own_process(|| {
        let game = launched("the-window-dpi", Screen::Fullscreen);

        // Every read orb made was on the far side of the call, and every one answered the panel's own
        // pixels. Not "the last one": a read before it is a layout laid out against two thirds of the
        // monitor, and it would not have to be the last read to do that.
        let reads = game.sim().windows().monitor_reads();
        assert!(!reads.is_empty(), "orb never read the monitor at all");
        assert!(
            reads.iter().all(|(aware, _)| *aware),
            "orb read the monitor before saying it was DPI aware: {reads:?}"
        );
        assert!(
            reads
                .iter()
                .all(|(_, rect)| (rect.width(), rect.height()) == REAL),
            "a read did not answer the panel's real pixels: {reads:?}"
        );
        // Which is a different size from the one the same panel answers before the call, and that is
        // what makes the ordering worth asserting rather than a formality.
        assert_ne!(SCALED, REAL);

        // And orb said so, which is the line somebody reading a launch's log has.
        assert!(
            game.log()
                .said("screen: display scaling is being ignored, sizes are real pixels"),
            "the log does not say scaling was turned off:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// A host that refuses it leaves every size scaled, and orb says which of the two happened.
///
/// The other half of the measurement above, and the reason the log line has two spellings: a launch
/// whose sizes are two thirds of what was asked for reads as a fault, and this line is what says it was
/// the host's answer rather than orb's arithmetic. The monitor then reads 2560x1440 for the whole launch
/// and the fullscreen window covers that.
#[test]
fn a_refused_dpi_awareness_leaves_the_sizes_scaled_and_says_so() {
    in_its_own_process(|| {
        let game = Fake::attach_to_a_panel(
            Panel {
                refuses_dpi_awareness: true,
                ..Panel::scaled()
            },
            "the-window-dpi-refused",
            the_run(),
            |config| config.screen = Screen::Fullscreen,
        );
        game.creates_its_window();

        let made = the_window(&game);
        assert_eq!(
            (made.asked.width(), made.asked.height()),
            SCALED,
            "the window covered the panel's real pixels on a host that refused to report them"
        );
        assert!(
            game.log().said(
                "screen: display scaling could not be turned off; sizes are whatever Windows scales \
                 them to"
            ),
            "the log does not say the call was refused:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Borderless fullscreen keeps the aspect ratio and blacks the rest, with no frame to remove.
///
/// Measured: `screen: fullscreen — window at 0,0 sized 3840x2160, client 3840x2160` on that monitor.
/// The game's own `CreateWindowExA` arguments are rewritten, so there is no frame to take off and
/// nothing flashes first.
#[test]
fn borderless_fullscreen_fills_the_monitor_with_no_frame_and_no_flash() {
    in_its_own_process(|| {
        let game = launched("the-window-fullscreen", Screen::Fullscreen);
        let made = the_window(&game);

        // Both ways: the window covers the monitor and so does the client, which is the borderless part
        // — a window with a frame would have one of the two smaller than the other.
        assert_eq!((made.asked.left, made.asked.top), (0, 0));
        assert_eq!((made.asked.width(), made.asked.height()), REAL);
        assert_eq!((made.client.width(), made.client.height()), REAL);
        assert!(
            !made.framed,
            "a borderless window was asked for with a style that has a frame"
        );

        // And nothing flashed first, which is what `the_window` counting one window says: the size is
        // decided inside the game's own creating call, so there is no window of another size on the
        // screen before it and nothing to move afterwards.
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
            "the game is not its own ratio inside the monitor"
        );
        assert_eq!(game_at.left, 480);
        assert_eq!(REAL.0 - game_at.right, 480);
    });
}

/// A chosen size is centred exactly, and the status line gets the black either side of a 4:3 game.
///
/// Measured: `screen: 2560x1440 — window at 637,340 sized 2566x1480, client 2560x1440`, centred
/// exactly — **(3840−2566)/2 = 637** and **(2160−1480)/2 = 340**. The game is letterboxed to
/// **1920x1440** inside that client, **320 pixels either side**, and the `no black to write in` line a
/// 4:3 client produces never appeared.
#[test]
fn a_chosen_size_is_centred_and_leaves_the_status_line_its_black() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-centred",
            Screen::Window {
                width: 2560,
                height: 1440,
            },
        );
        let made = the_window(&game);

        assert_eq!((made.asked.width(), made.asked.height()), (2566, 1480));
        assert_eq!((made.client.width(), made.client.height()), (2560, 1440));
        assert_eq!((made.asked.left, made.asked.top), (637, 340));
        assert_eq!(
            (made.asked.left, made.asked.top),
            ((REAL.0 - 2566) / 2, (REAL.1 - 1480) / 2)
        );
        assert!(
            game.log()
                .said("screen: 2560x1440 — window at 637,340 sized 2566x1480, client 2560x1440"),
            "the log does not say what window came out:\n  {}",
            game.log().lines().join("\n  ")
        );

        // 4:3 inside a 16:9 client of 1440 rows is 1920 wide, so 320 pixels of black either side — which
        // is where the status line goes.
        let game_at =
            unsafe { orb_core::window::letterbox() }.expect("a letterbox for a window that exists");
        assert_eq!(
            (game_at.left, game_at.top, game_at.right, game_at.bottom),
            (320, 0, 2240, 1440)
        );
        assert_eq!(game_at.left, 320);
        assert_eq!(made.client.right - game_at.right, 320);

        // And that black is room enough to write in, which the line orb writes when there is none is the
        // other half of: 320 pixels is far past the smallest readable text, so the run gets a status
        // line rather than the `no black to write in` line a 4:3 client produces.
        unsafe { orb_core::window::write_beside(&[String::from("CH 05"), String::from("HOLD")]) };
        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("no black to write in")),
            "orb found nowhere to write beside a game with 320 pixels of black either side:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// The stack goes in the bar down the side, aligned on the game's own edge and stacked up from the
/// bottom corner — and a shorter stack afterwards clears the rows the longer one wrote in.
///
/// **The clearing is a bug that was fixed and had never had a test.** What a stack of fewer lines or a
/// smaller font repaints is its own rows; the rows above it are the ones the last stack wrote in, and a
/// bar that repainted only its own left the top line of the previous one on the screen for the rest of
/// the run.
///
/// The geometry is the client's: a 2560x1440 client letterboxes a 4:3 game to 1920x1440 at x 320, so the
/// bar is the 320 pixels from 2240 to 2560, the text starts a [`BAR_MARGIN`] inside it at 2248 and the
/// lowest line's bottom is a margin above the client's own at 1432. At 30 pixels of em a stack of two is
/// 60 rows, so the block runs from 1372.
#[test]
fn a_status_line_goes_down_the_side_of_the_game_and_clears_what_it_wrote_before() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-status-beside",
            Screen::Window {
                width: 2560,
                height: 1440,
            },
        );
        let window = the_window(&game).handle;

        unsafe { orb_core::window::write_beside(&[String::from("CH 05"), String::from("HOLD")]) };
        let two = written(&game);
        assert_eq!(two.window, window, "the lines went to another window");
        assert_eq!(two.lines, ["CH 05", "HOLD"]);
        assert_eq!(
            (two.bar.align, two.bar.x),
            (Bar::LEFT, 2240 + BAR_MARGIN),
            "the stack is not left-aligned a margin inside the bar down the side"
        );
        assert_eq!(two.bar.height, BAR_TEXT_HEIGHT, "the em was reduced");
        assert_eq!(two.bar.bottom, 1440 - BAR_MARGIN);
        assert_eq!(
            two.bar.area,
            Rect {
                left: 2240,
                top: 1432 - BAR_TEXT_HEIGHT * 2,
                right: 2560,
                bottom: 1432,
            },
            "the block is not the two lines in the black beside the game"
        );

        // One line where there were two: its own rows are 30 tall, and what is repainted is those and
        // the 30 above them that the line before it was in.
        unsafe { orb_core::window::write_beside(&[String::from("CH 06")]) };
        let one = written(&game);
        assert_eq!(one.lines, ["CH 06"]);
        assert_eq!(
            one.bar.area, two.bar.area,
            "a shorter stack left the row the longer one wrote in on the screen"
        );

        // And nothing at all is the bar cleared. The rows repainted are the ones the last stack's own
        // block was in and no more: the 30 above them were cleared by the call before this one, which is
        // what the rows it repainted were for.
        unsafe { orb_core::window::write_beside(&[]) };
        let none = written(&game);
        assert!(none.lines.is_empty());
        assert_eq!(
            none.bar.area,
            Rect {
                left: 2240,
                top: 1432 - BAR_TEXT_HEIGHT,
                right: 2560,
                bottom: 1432,
            },
            "the cleared rows are not the ones the line that was there was in"
        );
    });
}

/// A client taller than the game's ratio puts the bar under the game instead, and the text is aligned on
/// the game's own edge from the other side.
///
/// The two bars are one decision with the alignment as half of its answer: down the side the text runs
/// away from the game, and under it the text runs back towards the game's own right-hand edge — which is
/// what keeps it off the game either way.
///
/// A 1600x1400 client letterboxes a 4:3 game to 1600x1200 at y 100, so the black is the 100 rows from
/// 1300 down and the strip is the client's whole width. The stack is held inside that black: two lines at
/// 30 would start at 1332, which is below the game.
#[test]
fn a_status_line_under_the_game_is_aligned_on_the_games_own_edge() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-status-below",
            Screen::Window {
                width: 1600,
                height: 1400,
            },
        );
        let made = the_window(&game);
        assert_eq!((made.client.width(), made.client.height()), (1600, 1400));
        let game_at =
            unsafe { orb_core::window::letterbox() }.expect("a letterbox for a window that exists");
        assert_eq!(
            game_at,
            Rect {
                left: 0,
                top: 100,
                right: 1600,
                bottom: 1300
            }
        );

        unsafe { orb_core::window::write_beside(&[String::from("CH 05"), String::from("HOLD")]) };
        let bar = written(&game).bar;
        assert_eq!(
            (bar.align, bar.x),
            (Bar::RIGHT, 1600),
            "the stack under the game is not aligned on the game's own right-hand edge"
        );
        assert_eq!(
            bar.area,
            Rect {
                left: 0,
                top: 1392 - BAR_TEXT_HEIGHT * 2,
                right: 1600,
                bottom: 1392,
            },
            "the block is not the two lines in the black under the game"
        );
        assert!(
            bar.area.top >= game_at.bottom,
            "the block reaches {} rows into the game",
            game_at.bottom - bar.area.top,
        );
    });
}

/// A stack taller than the black under the game starts at the game's own bottom edge, and the lines that
/// do not fit are clipped there rather than drawn over it.
///
/// **This is the case a run is in and the one above is not.** `runtime::write_status` pushes a chapter's
/// name and its retry count only while a run is being chaptered, so a title menu is three lines and a run
/// is five — and five at 30 pixels is 150 into the 92 rows this client leaves. Watched on the real game at
/// the title menu, where three lines fit with two rows to spare; the five-line stack is what plays, and
/// what holds it off the game is the clamp here rather than anything a launch has been read for.
///
/// The lines themselves are only counted, so what they say is what a run's own are: the numbers are
/// `write_status`'s, and the clipping is a property of how many there are.
#[test]
fn a_stack_taller_than_the_black_under_the_game_starts_at_the_games_edge() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-status-clipped",
            Screen::Window {
                width: 1600,
                height: 1400,
            },
        );
        let game_at =
            unsafe { orb_core::window::letterbox() }.expect("a letterbox for a window that exists");

        let lines: Vec<String> = [
            "CH 05",
            "RETRY 2",
            "INPUT LAG 3.4ms",
            "COMPOSE 2.6ms",
            "59.9fps",
        ]
        .iter()
        .map(|line| String::from(*line))
        .collect();
        unsafe { orb_core::window::write_beside(&lines) };
        let bar = written(&game).bar;

        // Taller than the black it goes in, which is what this e2e test is about: without the clamp the
        // stack would start here, and that is inside the game.
        let unclamped = bar.bottom - bar.height * lines.len() as i32;
        assert!(
            unclamped < game_at.bottom,
            "{} lines at {} do not overrun the black, so nothing here is clamped",
            lines.len(),
            bar.height,
        );

        assert_eq!(
            bar.area.top, game_at.bottom,
            "the block does not start at the game's own bottom edge"
        );
        assert_eq!(
            bar.area,
            Rect {
                left: 0,
                top: 1300,
                right: 1600,
                bottom: 1392,
            },
            "the block is not the whole of the black under the game"
        );
        // And the lines are still written from the bottom up at their own height, so which of them is
        // clipped is the top of the stack and not all of it: the block is 92 rows and each line is 30.
        assert_eq!(bar.height, BAR_TEXT_HEIGHT, "the em was reduced");
        assert_eq!(bar.bottom, 1400 - BAR_MARGIN);
    });
}

/// A stack wider than the bar it goes in is written smaller, down to what is still readable.
///
/// Clipped at the bar's edge a line cannot be read at all, which is worse than small — so the em is
/// scaled by the ratio the widest line overruns by. How wide a line comes out is the font's business, so
/// this is the e2e test that declares one: at 100 pixels a character the five of `CH 05` are 500 wide in
/// 312 pixels of black, and 30 pixels of em scaled by that is 18.
#[test]
fn a_stack_too_wide_for_its_bar_is_written_smaller() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-status-shrunk",
            Screen::Window {
                width: 2560,
                height: 1440,
            },
        );
        game.sim().text().measures(
            BAR_TEXT_HEIGHT,
            Metric {
                advance: 100,
                line: 32,
            },
        );

        unsafe { orb_core::window::write_beside(&[String::from("CH 05")]) };
        let bar = written(&game).bar;
        assert_eq!(
            bar.height,
            BAR_TEXT_HEIGHT * 312 / 500,
            "the em was not scaled by what the widest line overran the black by"
        );
        assert!(
            game.log()
                .said("screen: 500px of text in 312px of black, writing at 18px"),
            "orb did not say it was writing smaller:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And a client of the game's own ratio really does produce that line, which is what makes its absence
/// above worth reading.
///
/// A 4:3 window is the game filling every pixel of its client: the letterbox is the whole of it, so
/// there is no black anywhere and nothing orb can write in. Measured only as arithmetic — nobody has
/// asked for a 4:3 window on this machine — so what is held is the line, and the numbers in it are the
/// client's own.
#[test]
fn a_four_three_client_leaves_no_black_and_orb_says_so() {
    in_its_own_process(|| {
        let game = launched(
            "the-window-no-black",
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
