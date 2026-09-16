//! The lives on the game's status panel, painted out for a run that cannot lose one.
//!
//! Dying in a pointdevice run costs the chapter and not a life: the menu goes up, and the snapshot
//! that puts the chapter back puts the count back with it. So the number in the panel decides
//! nothing, while being the one thing left on the screen still describing the game as it was.
//! th15 writes DISABLE across its own row of lives, which is where the mode comes from and so what
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
//!
//! **The game's own count of lives is painted over with a brush stroke in a pointdevice run**, with `DISABLE`
//! written on it. Dying there costs the chapter and not a life — the menu goes up, and the snapshot that puts
//! the chapter back puts the count back with it — so the row is the one thing left on the screen still
//! describing the game as it was. th15 says it the same way across its own row of lives, which is where
//! somebody who wants the mode will look for it. Drawn on three conditions: the mode, a run of somebody's
//! own rather than a demo or a replay, and a chapter with a snapshot to go back to — that last being the
//! frames at a stage's start before its own snapshot has been taken, where a death does still cost a life.
//!
//! Asked of the run and not of the frame, unlike the death, which is a comparison against the frame before
//! and belongs to one. A run passes through frames that are not gameplay frames and are drawn like any
//! other: the single frame a stage transition is built in, which is a quarter of a second of screen because
//! the game builds the next stage inside it, and the frame a chapter is put back on, where what orb knows of
//! the frame before has deliberately been dropped. So what decides is whether the run being tracked is one
//! somebody is playing — settled where that stage's snapshot was taken — and, past the end of the run,
//! whether the game is still painting that row at all. A mark that stopped on any of those frames would
//! leave whatever the game paints in that row standing there instead, which is the count: the game paints it
//! from its own "this row changed" bits, and does not ask orb first.
//!
//! **The stroke is a picture of one and not a generated one.** Generating one was tried first, and every
//! version read as a smear rather than as a brush: a spine with a taper has the ends wrong, and noise along
//! an edge does not carry ink the way a hair does. So the stroke is a real one, and what the tree carries is
//! the picture of it: `crates/orb-core/brush.png`. `crates/orb-core/build.rs` bakes that into the 144x30 of
//! coverage the drawing wants, one byte a pixel — how far each pixel of ink on paper is from the paper,
//! cropped to the ink, averaged by area and put through two smoothstep passes.
//!
//! The picture is grey, and that is a size and not a taste: only one channel of it is ever read, so the
//! colour one it was baked from gave the same 4320 bytes at three times the file. WebP is the wrong trade —
//! lossless came out *larger* than the grey PNG, 884KB against 460KB, and both of its modes want a decoder
//! that cannot be sixty lines. What made the difference was never the format.
//!
//! In the build rather than in a script anybody runs, and that is the whole of why: a script means its
//! output committed beside the picture it came from, which is two copies of one stroke with the editable one
//! the wrong one. Only the inflate comes from a crate — walking a PNG's chunks and undoing its row filters
//! is sixty lines that say what the picture is, against eight crates that would say it for every picture
//! there has ever been. The picture's ink is 4:1 and the squash to 144x30 is what fits a stroke between the
//! score's row and the bombs'.
//!
//! **The count is not painted out; the stroke goes over it.** Where the ink is dry, the stars show faintly
//! through — which is what they are, disabled rather than gone, and one gained still shows. That needs the
//! count drawn again underneath every frame, and the game is asked to do it: `Gui`'s `flags.flag0`, the two
//! bits set by whatever changes the count and taken one off by each draw, is written back to 2 before the
//! game draws. So the game erases that row and draws its own stars again for orb, background and all.
//!
//! **Which means every frame the game paints that row is a frame the mark has to go over.** The row it
//! paints last is the one left standing on the screen, and a run's panel outlives the run: leaving one ends
//! it on a single frame and the panel stays there until the front end has drawn its own screen. So what ends
//! the mark is the game no longer painting that row — its `Gui`'s own draw job gone from the draw chain —
//! and not the run ending. Ending it with the run left the count back on the panel for the whole fade to
//! the title.
//!
//! The two strips the stroke reaches past that row are orb's to paint, and painted first: a panel that is
//! not repainted is one where a mark blended over what the last frame left hardens into its own edges
//! within a second. They are painted with the game's own panel tile — `front.anm`'s 32x32 at (0, 224), laid
//! on the grid it lays it on, from the texture the game has loaded in slot 13 — rather than with a colour of
//! orb's, because the panel is a noise and a flat rectangle inside it reads as a patch. What that also buys:
//! what is left where orb stops drawing is the panel the game would have painted there itself.
//!
//! **How far past the row the stroke may reach is the game's to say, and can be nothing.** th06's panel
//! leaves 8 pixels either side of that row, which the squash to 144x30 was made to fit inside; th07 puts its
//! bombs 16 pixels under a row 16 tall, so there the box is the row, the word is scaled to fit it, and there
//! are no strips to paint at all — a box hung 7 pixels into the bombs' row took the tops of the bombs' own
//! stars with it.
//!
//! **Over the count and not over the `Player` beside it.** The count is where the stars are — from (496,
//! 122) rightwards, 16 apart, and eight of them at the most, which is 496 to the right edge of the output —
//! and it is a row the game will repaint on being asked. The label is drawn when the stage begins and never
//! again, so a mark over that one would still be on the screen after the run it belonged to. What repaints
//! what, and how seldom, is a fact about the game: `Gui::OnDraw` erases and redraws a row of the panel only
//! while that row's two bits are set, and the background behind the whole panel only for the first 250
//! frames of a stage, that being where its own script reaches `ExitHide`. Nothing repaints any of it after
//! that unless the game is told to clear the back buffer, which is the setting orb reads for
//! `clears_back_buffer`.

use crate::game::{PanelTile, Rect};
use crate::overlay::{Frame, Label, Overlay, Picture};
use orb_api::Mask;

use crate::{brush, log};

/// The panel's own average colour, for painting out the stars where the game's sheet is not there
/// to paint with. th06's panel is a noise of four shades within seven of each other, so the
/// average is the nearest one flat colour there is to it.
const PANEL: u32 = 0xff2a_2a39;
/// The stroke's own ink, which in the picture it was baked from is very nearly black.
const INK: u32 = 0xff0a_0a0c;
const WORD: u32 = 0xffff_ffff;

const TEXT: &str = "DISABLE";

/// How far past the word its own drop shadow reaches, which `Frame::label` draws a pixel down and right of it:
/// the word's box has to leave that pixel where the box is the row and no more.
const SHADOW: f32 = 1.0;

pub struct LivesMark {
    stroke: Picture,
    word: Label,
    /// Whether the log has been told what the strips are painted with.
    reported: bool,
}

/// Beside `new` because the module is public now, and one with nothing baked or chosen yet is exactly
/// what `new` makes.
impl Default for LivesMark {
    fn default() -> Self {
        Self::new()
    }
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
    pub unsafe fn draw(
        &mut self,
        overlay: &Overlay,
        row: Rect,
        room: f32,
        panel: Option<PanelTile>,
    ) {
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
        let over = brush_area(row, room);
        unsafe { paint_panel(&frame, over, row, panel) };
        frame.picture_in(
            &self.stroke,
            over.left,
            over.top,
            over.width,
            over.height,
            INK,
        );

        // The word scaled to the box where the box is the shorter of the two, which is a panel with no room
        // either side of the row: a word drawn at the size its face was baked at would hang into the rows the
        // stroke has just been kept out of. One pixel of the box is the drop shadow's, which is drawn a pixel
        // down and right of the word itself.
        let scale = ((over.height - SHADOW) / self.word.height()).min(1.0);
        let word = (self.word.width() * scale, self.word.height() * scale);
        let (x, y) = word_at(row, over, word);
        frame.label_scaled(&self.word, x, y, scale, WORD);
    }
}

/// Where the stroke goes: as wide as the count's row, taller than it where the panel has room, and centred
/// on it.
///
/// Taller because the picture is a stroke and a stroke is not the shape of a row — 4:1 in the
/// picture against the row's 9:1. **What it may not do is reach the rows either side**, which the
/// game repaints no oftener than it repaints this one, so how far past the row it may go is
/// `room` — see `Game::room_around_the_lives_row`. th06 has 8 pixels there and the stroke's own
/// box wants 7; th07 has none, and the box is the row.
fn brush_area(row: Rect, room: f32) -> Rect {
    let width = brush::WIDTH as f32;
    let height = brush::HEIGHT as f32;
    let top = (row.center_y() - height / 2.0).round().max(row.top - room);
    let bottom = (top + height).min(row.top + row.height + room);
    Rect {
        left: row.left,
        top,
        width,
        height: bottom - top,
    }
}

/// Where the word goes: centred on the stroke across, and on the count's own row down, so that it
/// sits where the number it replaces sat.
///
/// Held inside the box, the drop shadow's own pixel included: centring an odd word on an even row
/// rounds it a pixel down, and where the box *is* the row that pixel of shadow is in the row below.
fn word_at(row: Rect, over: Rect, word: (f32, f32)) -> (f32, f32) {
    let (width, height) = word;
    let y = (row.center_y() - height / 2.0)
        .round()
        .min(over.top + over.height - height - SHADOW)
        .max(over.top);
    ((over.left + (over.width - width) / 2.0).round(), y)
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
/// not line up with the row: th06's runs from 416 in 32s and the count starts at 496.
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
    use super::{LivesMark, SHADOW, brush_area, strips, word_at};
    use crate::brush;
    use crate::game::Rect;
    use crate::overlay::Drawing;
    use orb_sim::{Drawn, Quad};

    /// th06's own row: the bar the game erases the count with, which is where the mark goes.
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
    /// How much panel that game leaves either side of the row, which is what it answers for
    /// `Game::room_around_the_lives_row`: the nearer neighbour is the bombs', 8 pixels under.
    const ROOM: f32 = 8.0;

    /// The stroke covers the whole count and stays out of the rows either side. Every star the
    /// player can have is inside it: eight of them, 16 apart from 496, is 624.
    #[test]
    fn the_stroke_covers_the_count_and_nothing_else() {
        let over = brush_area(ROW, ROOM);
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
        let over = brush_area(ROW, ROOM);
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
        let over = brush_area(ROW, ROOM);
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

    /// And a panel with no room either side of the row — th07's, whose bombs are 16 pixels under a row
    /// 16 tall — gives the stroke that row and nothing else, so there is nothing beside it to paint and
    /// the word has the row to fit in.
    #[test]
    fn a_panel_with_no_room_gives_the_stroke_the_row_and_no_more() {
        let over = brush_area(ROW, 0.0);
        assert_eq!((over.top, over.height), (ROW.top, ROW.height));
        let [above, below] = strips(over, ROW);
        assert_eq!((above.height, below.height), (0.0, 0.0));
        // The word inside it, drop shadow and all: what it is scaled by is the box less that pixel.
        let word = (63.0, 25.0);
        let scale = (over.height - SHADOW) / word.1;
        let (_, y) = word_at(ROW, over, (word.0 * scale, word.1 * scale));
        assert!(y >= ROW.top, "{y}");
        assert!(
            y + word.1 * scale + SHADOW <= ROW.top + ROW.height,
            "{}",
            y + word.1 * scale + SHADOW
        );
    }

    /// The picture is the size the row is drawn from, which is what `brush_area` assumes.
    #[test]
    fn the_brush_is_the_size_it_says() {
        assert_eq!(
            brush::COVERAGE.len(),
            (brush::WIDTH * brush::HEIGHT) as usize
        );
    }

    /// The mark, drawn through the real overlay onto a device that keeps what it was asked for.
    ///
    /// The row is th06's own. No panel tile, which is the case the log warns about — the strips
    /// either side of the stroke are painted flat — and it is the one a test can set up without a
    /// game's sprite sheet.
    fn drawn_mark(row: Rect) -> Drawn {
        let drawing = Drawing::new();
        let mut mark = LivesMark::new();
        drawing.drawn(|overlay| unsafe { mark.draw(overlay, row, ROOM, None) })
    }

    fn as_quad(rect: Rect) -> Quad {
        Quad {
            x: rect.left,
            y: rect.top,
            width: rect.width,
            height: rect.height,
            color: 0,
            texture: 0,
        }
    }

    /// What the mark is for, asked of the frame rather than of the arithmetic: something is drawn
    /// over every part of the count's row, and nothing is drawn over the rows either side.
    ///
    /// The rows either side are the assertion that matters. The stroke is taller than the row it
    /// goes over — a stroke is not the shape of a row — so the one thing that can go wrong is it
    /// reaching the bombs eight pixels below or the score above, and neither row is repainted by the
    /// game often enough to recover.
    #[test]
    fn the_mark_covers_the_count_and_reaches_neither_row_beside_it() {
        let quads = drawn_mark(ROW).quads;
        assert!(!quads.is_empty(), "the mark drew something");

        let row = as_quad(ROW);
        assert!(
            quads.iter().any(|quad| quad.covers(&row)),
            "the count's row is covered: {quads:?}",
        );

        let bombs = Quad {
            y: BOMB_ROW_TOP,
            ..row
        };
        let score = Quad {
            y: SCORE_ROW_BOTTOM - ROW.height,
            ..row
        };
        for (name, beside) in [("the bombs", bombs), ("the score", score)] {
            assert!(
                !quads.iter().any(|quad| quad.overlaps(&beside)),
                "{name} is left alone: {quads:?}",
            );
        }
    }

    /// The count is drawn again underneath the mark and the stroke is blended over it, so the stars
    /// show through where the ink is dry. What carries that is the picture, not the colour: the ink
    /// is opaque and the brush's own coverage is the alpha it is modulated by.
    ///
    /// So what is worth pinning is that the quad over the row goes through a texture of its own
    /// rather than the overlay's white texel. Through the white one the same opaque ink would be a
    /// flat patch over the row, which is the one thing this mark must not look like — and it is a
    /// one-line mistake away, since every solid fill in the overlay is drawn that way.
    #[test]
    fn the_stroke_over_the_row_is_a_picture_and_not_a_flat_fill() {
        let drawn = drawn_mark(ROW);
        let row = as_quad(ROW);

        let over_the_row: Vec<_> = drawn
            .quads
            .iter()
            .filter(|quad| quad.covers(&row))
            .collect();
        assert!(!over_the_row.is_empty(), "something covers the row");

        let pictured = drawn.pictured();
        for quad in over_the_row {
            assert!(
                pictured.contains(quad),
                "what covers the row is a picture: {quad:?}",
            );
            // And it is the ink, so that a stroke drawn in the wrong colour is not read as one.
            assert_eq!(quad.color, super::INK);
        }
    }
}
