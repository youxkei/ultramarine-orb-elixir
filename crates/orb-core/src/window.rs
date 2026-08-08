//! How much of the screen the game gets, worked out.
//!
//! The window is the size it is going to be from the moment it exists, because the arguments of the
//! game's own `CreateWindowExA` call are rewritten on the way through — there is no frame to remove
//! afterwards and nothing to flash on screen first. Fullscreen is that window borderless and covering the
//! monitor; a size is that window centred on it, with a caption and nothing to drag, since the size is a
//! setting rather than something to pull about.
//!
//! In windowed mode the game asks Direct3D for a `D3DSWAPEFFECT_COPY` swap chain, and that is the swap
//! effect that honours a destination rectangle on `Present`. So the 640x480 back buffer is presented into
//! a centred rectangle with the game's aspect ratio, and the rest of the window keeps the black background
//! its class was given.
//!
//! **Getting in front of the calls is `orb::window`**: the two imports, the `Present` slot, the black
//! brush and the lines written in the letterbox. This is the half that decides the rectangles, which is
//! the half a test can hold to a number.

use orb_api::Rect;
use orb_config::Screen;

/// All a borderless window needs. Anything else — caption, frame, system menu — is what puts a border on
/// it.
///
/// `WS_POPUP | WS_VISIBLE`, written out rather than taken from `windows-sys`: what a style is is the
/// game's own `CreateWindowExA` argument, and this is the half that decides it. `orb::window` holds the
/// two against Windows' own numbers at compile time, so a wrong one here is a build that stops.
pub const BORDERLESS_STYLE: u32 = 0x8000_0000 | 0x1000_0000;
/// A window of a chosen size: a caption to move it by and a system menu to close it with, and nothing to
/// resize it with. The size is one of the settings, so dragging the edge of the window would be a second
/// place to say it and the one that is not written down.
///
/// `WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE`.
pub const WINDOWED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x0002_0000 | 0x1000_0000;

/// A window to create: what kind it is, and the whole of it — frame included — in the monitor's
/// own coordinates.
pub struct Placed {
    pub style: u32,
    pub area: Rect,
}

/// Where the game's window goes.
///
/// The size in the settings is the size of what is *inside* the window, so that `1280x720` is
/// 1280x720 of game however thick this machine's window frames are; `AdjustWindowRect` is what
/// turns that into the window to ask for.
pub fn placed(monitor: Rect, screen: Screen) -> Placed {
    let Screen::Window { width, height } = screen else {
        return Placed {
            style: BORDERLESS_STYLE,
            area: monitor,
        };
    };
    let client = Rect::sized(width as i32, height as i32);
    // A failure leaves the rectangle as the client area, which is a window a frame too small
    // rather than no window at all.
    let area = orb_api::window::adjust_window_rect(client, WINDOWED_STYLE, false).unwrap_or(client);
    Placed {
        style: WINDOWED_STYLE,
        area: centred(monitor, area),
    }
}

/// A rectangle of that size in the middle of the monitor, and against its top-left corner if it
/// is too big to be in the middle of it — a window whose caption is off the top of the screen
/// cannot be moved back on.
pub fn centred(monitor: Rect, area: Rect) -> Rect {
    let (width, height) = (area.width(), area.height());
    let left = monitor.left + ((monitor.width() - width) / 2).max(0);
    let top = monitor.top + ((monitor.height() - height) / 2).max(0);
    Rect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

/// The largest rectangle with `content`'s aspect ratio that fits `client`, centred.
pub fn fit(client: Rect, content: (u32, u32)) -> Rect {
    let available_width = i64::from(client.width());
    let available_height = i64::from(client.height());
    let wanted_width = i64::from(content.0.max(1));
    let wanted_height = i64::from(content.1.max(1));

    // Whichever axis runs out first sets the scale. All integer, so the result is
    // exactly the game's ratio rather than nearly it.
    let (width, height) = if available_width * wanted_height <= available_height * wanted_width {
        (
            available_width,
            available_width * wanted_height / wanted_width,
        )
    } else {
        (
            available_height * wanted_width / wanted_height,
            available_height,
        )
    };

    let left = client.left + ((available_width - width) / 2) as i32;
    let top = client.top + ((available_height - height) / 2) as i32;
    Rect {
        left,
        top,
        right: left + width as i32,
        bottom: top + height as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::{centred, fit};
    use orb_api::Rect;

    fn client(width: i32, height: i32) -> Rect {
        Rect::sized(width, height)
    }

    /// A window goes in the middle of the monitor it is on, which for a second monitor is not
    /// the middle of anything measured from zero.
    #[test]
    fn a_window_is_centred_on_its_monitor() {
        let placed = centred(client(1920, 1080), client(1280, 760));
        assert_eq!(
            (placed.left, placed.top, placed.right, placed.bottom),
            (320, 160, 1600, 920)
        );
        let second = Rect {
            left: 1920,
            top: 0,
            right: 3840,
            bottom: 1080,
        };
        let placed = centred(second, client(1280, 760));
        assert_eq!((placed.left, placed.top), (2240, 160));
    }

    /// Against the corner rather than half off the top, since a caption above the screen cannot
    /// be dragged back onto it.
    #[test]
    fn a_window_too_big_for_the_monitor_starts_at_its_corner() {
        let placed = centred(client(800, 600), client(1280, 760));
        assert_eq!((placed.left, placed.top), (0, 0));
        assert_eq!((placed.right, placed.bottom), (1280, 760));
    }

    #[test]
    fn a_four_three_game_is_pillarboxed_on_a_sixteen_nine_screen() {
        let rect = fit(client(2560, 1440), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (320, 0, 2240, 1440)
        );
    }

    #[test]
    fn a_game_wider_than_the_screen_is_letterboxed() {
        let rect = fit(client(1000, 1000), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (0, 125, 1000, 875)
        );
    }

    #[test]
    fn a_matching_ratio_fills_the_screen() {
        let rect = fit(client(1600, 1200), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (0, 0, 1600, 1200)
        );
    }
}
