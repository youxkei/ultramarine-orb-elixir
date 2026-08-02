//! The lives on the game's status panel, painted out for a run that cannot lose one.
//!
//! Dying in a pointdevice run costs the chapter and not a life: the menu goes up, and the snapshot
//! that puts the chapter back puts the count back with it. So the number in the panel decides
//! nothing, while being the one thing left on the screen still describing the game as it was.
//! 紺珠伝 writes DISABLE across its own 残機 row, which is where the mode comes from and so what
//! somebody who wants it will look for.
//!
//! **The stroke is a picture of a brush stroke, not a generated one.** Generating one was tried
//! first — a spine, a taper, edges wandering on noise, dry bristles thresholded out of an ink
//! field — and every version of it read as a smear rather than as a brush: the ends wrong, the
//! carry of the ink wrong, the hairs too even. So `brush.rs` is a real stroke baked down to
//! coverage by `build.rs`, out of the picture beside it — and this draws that. The picture is the
//! only copy of the stroke in the tree, which is what putting the bake in the build is for.
//!
//! **The count is not painted out.** The stroke goes over it, and where the ink is dry the stars
//! show faintly through — which is what they are: disabled, not gone, and one gained still shows.
//! That needs the count drawn again under the ink every frame, which is what
//! `Game::repaint_lives_row` asks the game for.
//!
//! **What that leaves is the two strips the stroke reaches past that row**, above and below, which
//! are panel the game repaints only for the first 250 frames of a stage. A mark blended over what
//! the last frame left there would harden into its own edges within a second, so those two are
//! painted first — with the game's own panel tile rather than a colour of orb's, because the panel
//! is a noise and a flat rectangle inside it reads as a patch. Painting with the tile also means
//! that what is left there when orb stops drawing is the panel the game would have painted.

use crate::game::{PanelTile, Rect};
use crate::overlay::{Frame, Label, Overlay, Picture};
use crate::text::Mask;
use crate::{brush, log};

/// The panel's own average colour, for painting out the stars where the game's sheet is not there
/// to paint with. 紅魔郷's panel is a noise of four shades within seven of each other, so the
/// average is the nearest one flat colour there is to it.
const PANEL: u32 = 0xff2a_2a39;
/// The stroke's own ink, which in the picture it was baked from is very nearly black.
const INK: u32 = 0xff0a_0a0c;
const WORD: u32 = 0xffff_ffff;

const TEXT: &str = "DISABLE";

pub struct LivesMark {
    stroke: Picture,
    word: Label,
    /// Whether the log has been told what the strips are painted with.
    reported: bool,
}

impl LivesMark {
    pub const fn new() -> Self {
        Self {
            stroke: Picture::new(),
            word: Label::new(),
            reported: false,
        }
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay, row: Rect, panel: Option<PanelTile>) {
        // Both baked before the overlay's frame is opened, the way the menus do it: baking creates
        // a texture and locks it, and the frame is a window with the device's state swapped out.
        if !self.stroke.baked() {
            unsafe { self.stroke.bake(overlay, &stroke_mask()) };
            log::line(&format!(
                "lives: the brush is {}x{} and {}",
                brush::WIDTH,
                brush::HEIGHT,
                if self.stroke.baked() {
                    "baked"
                } else {
                    "not baked; the row will be painted out and left at that"
                }
            ));
        }
        unsafe { self.word.set_in(overlay, overlay.mark_font(), TEXT) };
        // Said once, because the difference is visible and was not: without the game's own tile the
        // strips either side of the stroke are a flat colour, which is exactly the patch this is
        // meant not to be.
        if !std::mem::replace(&mut self.reported, true) {
            log::line(match panel {
                Some(_) => "lives: the panel's own tile is what the strips are painted with",
                None => {
                    "lives: no panel tile; the strips are painted flat and will show as a patch"
                }
            });
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };

        // The strips above and below the count's own row, which nothing else repaints.
        let over = brush_area(row);
        unsafe { paint_panel(&frame, over, row, panel) };
        frame.picture(&self.stroke, over.left, over.top, INK);

        let (x, y) = word_at(row, over, (self.word.width(), self.word.height()));
        frame.label(&self.word, x, y, WORD);
    }
}

/// Where the stroke goes: as wide as the count's row, taller than it, and centred on it.
///
/// Taller because the picture is a stroke and a stroke is not the shape of a row — 4:1 in the
/// picture against the row's 9:1. What it may not do is reach the row below: the bombs are 8
/// pixels under this one and the game repaints their row no oftener than it repaints this one.
fn brush_area(row: Rect) -> Rect {
    let width = brush::WIDTH as f32;
    let height = brush::HEIGHT as f32;
    Rect {
        left: row.left,
        top: (row.center_y() - height / 2.0).round(),
        width,
        height,
    }
}

/// Where the word goes: centred on the stroke across, and on the count's own row down, so that it
/// sits where the number it replaces sat.
fn word_at(row: Rect, over: Rect, word: (f32, f32)) -> (f32, f32) {
    let (width, height) = word;
    (
        (over.left + (over.width - width) / 2.0).round(),
        (row.center_y() - height / 2.0).round(),
    )
}

/// The stroke, as a coverage mask the overlay can bake.
fn stroke_mask() -> Mask {
    Mask {
        width: brush::WIDTH,
        height: brush::HEIGHT,
        // The overlay's masks are white with the coverage in the alpha, which is what a font
        // hands over and what the vertex colour is then modulated through.
        pixels: brush::COVERAGE
            .iter()
            .map(|coverage| u32::from(*coverage) << 24 | 0x00ff_ffff)
            .collect(),
    }
}

/// Paints the parts of `area` outside `row` with the panel's own background, tile by tile on the
/// game's grid.
///
/// Outside `row` only: the row itself is the game's to repaint, count and all — see
/// `Game::repaint_lives_row` — and painting the panel over it would take the count away, which is
/// the one thing this mark is not for.
///
/// Each tile is clipped to what is being painted, texture coordinates and all, since the grid does
/// not line up with the row: 紅魔郷's runs from 416 in 32s and the count starts at 496.
///
/// # Safety
/// Must run inside `frame`, with `panel` naming a live texture of the game's device.
unsafe fn paint_panel(frame: &Frame, area: Rect, row: Rect, panel: Option<PanelTile>) {
    for strip in strips(area, row) {
        unsafe { paint_strip(frame, strip, panel.as_ref()) };
    }
}

/// The parts of the stroke's box that are above and below the count's row. Either can be empty,
/// which is a box no taller than the row.
fn strips(area: Rect, row: Rect) -> [Rect; 2] {
    let above = (row.top - area.top).max(0.0);
    let below = (area.top + area.height - (row.top + row.height)).max(0.0);
    [
        Rect {
            left: area.left,
            top: area.top,
            width: area.width,
            height: above,
        },
        Rect {
            left: area.left,
            top: row.top + row.height,
            width: area.width,
            height: below,
        },
    ]
}

/// # Safety
/// Must run inside `frame`, with `panel` naming a live texture of the game's device.
unsafe fn paint_strip(frame: &Frame, area: Rect, panel: Option<&PanelTile>) {
    if area.height <= 0.0 {
        return;
    }
    let Some(panel) = panel else {
        frame.fill(area.left, area.top, area.width, area.height, PANEL);
        return;
    };
    let [u0, v0, u1, v1] = panel.uv;
    let first =
        |start: f32, origin: f32| origin + ((start - origin) / panel.pitch).floor() * panel.pitch;
    let mut y = first(area.top, panel.origin.1);
    while y < area.top + area.height {
        let mut x = first(area.left, panel.origin.0);
        while x < area.left + area.width {
            let left = x.max(area.left);
            let top = y.max(area.top);
            let right = (x + panel.pitch).min(area.left + area.width);
            let bottom = (y + panel.pitch).min(area.top + area.height);
            let across = |from: f32, to: f32, at: f32| from + (to - from) * at;
            unsafe {
                frame.piece(
                    panel.texture,
                    left,
                    top,
                    right - left,
                    bottom - top,
                    [
                        across(u0, u1, (left - x) / panel.pitch),
                        across(v0, v1, (top - y) / panel.pitch),
                        across(u0, u1, (right - x) / panel.pitch),
                        across(v0, v1, (bottom - y) / panel.pitch),
                    ],
                    0xffff_ffff,
                )
            };
            x += panel.pitch;
        }
        y += panel.pitch;
    }
}

#[cfg(test)]
mod tests {
    use super::{brush_area, strips, word_at};
    use crate::brush;
    use crate::game::Rect;

    /// 紅魔郷's own row: the bar the game erases the count with, which is where the mark goes.
    const ROW: Rect = Rect {
        left: 496.0,
        top: 122.0,
        width: 144.0,
        height: 16.0,
    };
    /// The row under it, whose bombs the stroke may not reach.
    const BOMB_ROW_TOP: f32 = 146.0;
    /// And the row above, whose score it may not reach either. It ends 16 below where it starts.
    const SCORE_ROW_BOTTOM: f32 = 98.0;

    /// The stroke covers the whole count and stays out of the rows either side. Every star the
    /// player can have is inside it: eight of them, 16 apart from 496, is 624.
    #[test]
    fn the_stroke_covers_the_count_and_nothing_else() {
        let over = brush_area(ROW);
        assert_eq!(over.left, ROW.left);
        assert!(over.left + over.width >= 624.0 + 16.0);
        assert!(over.top > SCORE_ROW_BOTTOM, "{}", over.top);
        assert!(
            over.top + over.height <= BOMB_ROW_TOP,
            "{}",
            over.top + over.height
        );
    }

    /// The word sits where the count sat: centred on the stroke across and on the row down, so it
    /// reads as what replaced the number rather than as something floating over the panel.
    #[test]
    fn the_word_sits_where_the_count_sat() {
        let over = brush_area(ROW);
        let word = (63.0, 25.0);
        let (x, y) = word_at(ROW, over, word);
        let before = x - over.left;
        let after = over.left + over.width - (x + word.0);
        assert!((before - after).abs() <= 1.0, "{before} against {after}");
        assert!((y + word.1 / 2.0 - ROW.center_y()).abs() <= 1.0);
        // Whole pixels, both of them: a label is a texture drawn through a linear filter, and half
        // a pixel is a blurred word.
        assert_eq!((x.fract(), y.fract()), (0.0, 0.0));
    }

    /// What gets painted is the two strips outside the count's row and not the row itself, which
    /// is the game's to repaint — with the count in it, which is the whole point.
    #[test]
    fn the_row_itself_is_left_to_the_game() {
        let over = brush_area(ROW);
        let [above, below] = strips(over, ROW);
        assert!(above.top + above.height <= ROW.top, "above the count");
        assert!(below.top >= ROW.top + ROW.height, "below it");
        // And between them they cover everything of the box that is not the row.
        assert_eq!(above.height + ROW.height + below.height, over.height);
        for strip in [above, below] {
            assert_eq!((strip.left, strip.width), (over.left, over.width));
        }
    }

    /// A box no taller than the row has nothing to paint, and a strip of no height paints nothing.
    #[test]
    fn a_box_inside_the_row_paints_nothing() {
        let [above, below] = strips(ROW, ROW);
        assert_eq!((above.height, below.height), (0.0, 0.0));
    }

    /// The picture is the size the row is drawn from, which is what `brush_area` assumes.
    #[test]
    fn the_brush_is_the_size_it_says() {
        assert_eq!(
            brush::COVERAGE.len(),
            (brush::WIDTH * brush::HEIGHT) as usize
        );
    }
}
