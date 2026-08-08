//! How the three questions orb puts on the screen are drawn: a list of items with a cursor on one
//! of them, over a wash of what is underneath.
//!
//! Here rather than three times over for the same reason as the keys it re-exports below — the
//! corrections landed here. What each question *decides* is [`orb_core::menu`], apart from this
//! because a decision is a function of a keyboard and a pad, both of which a test can hand over,
//! and a `Label` is not.

use crate::overlay::{Frame, Label};

/// The keys these menus read, unchanged at every call site: `menu_ui::moved`, `menu_ui::Keys` and
/// the rest are what the three questions were written against, and which side of the seam they now
/// live on is not something those three have an opinion about.
pub use crate::menu::{By, Keys, Pressed, moved};

/// The wash over the play field, for the menu that appears where a chapter was lost: under it is
/// the frame the player died on, which is what the menu is read against.
pub const DIM_FIELD: u32 = 0xb400_0000;
/// And over the whole screen, for the questions asked over the game's own front end: what is
/// under those is not something to leave half readable.
pub const DIM_SCREEN: u32 = 0xc800_0000;
pub const NORMAL: u32 = 0xffff_ffff;
pub const SELECTED: u32 = 0xffff_e066;
/// The line under the items, which says what the one under the cursor means.
pub const ASIDE: u32 = 0xffb0_b0b0;

pub const LINE_HEIGHT: f32 = 24.0;

/// Between the cursor and the item it is on. Beside the item rather than in a column of its own,
/// the items being centred: a column would sit a different distance from each of them.
const CURSOR_GAP: f32 = 6.0;

/// A line centred on `center`, which is every line these menus draw.
pub fn centred(frame: &Frame, label: &Label, center: f32, y: f32, colour: u32) {
    frame.label(label, center - label.width() / 2.0, y, colour);
}

/// The items, with the cursor on one of them. Returns the y the line after them goes on, since
/// what follows differs from one menu to the next.
pub fn list(
    frame: &Frame,
    labels: &[Label],
    cursor: &Label,
    center: f32,
    top: f32,
    selection: usize,
) -> f32 {
    let mut y = top;
    for (index, label) in labels.iter().enumerate() {
        let chosen = index == selection;
        centred(
            frame,
            label,
            center,
            y,
            if chosen { SELECTED } else { NORMAL },
        );
        if chosen {
            let x = center - label.width() / 2.0;
            frame.label(cursor, x - cursor.width() - CURSOR_GAP, y, SELECTED);
        }
        y += LINE_HEIGHT;
    }
    y
}
