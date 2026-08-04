//! What the three questions orb puts on the screen have in common.
//!
//! Each of them is up on frames the game is frozen on, which means its own input handling is not
//! running either — so each reads the keyboard itself and takes the pad from the sample orb's own
//! thread keeps. Without that second half a pad does nothing at all on one of these while working
//! perfectly on the game's own menu a keypress earlier, which is the whole of how it looks broken.
//!
//! And each is a list of items with a cursor on one of them, over a wash of what is underneath.
//!
//! Here rather than three times over because this is where the corrections landed: the pad's edge,
//! the frames a menu holds its keys off for, which button cancels, and which hand answered were
//! each found once and had to be fixed in three places.

use std::fmt;

use crate::game::Pad;
use crate::input::Keyboard;
use crate::overlay::{Frame, Label};

const VK_RETURN: u8 = 0x0d;
const VK_ESCAPE: u8 = 0x1b;
const VK_UP: u8 = 0x26;
const VK_DOWN: u8 = 0x28;
const VK_X: u8 = 0x58;
const VK_Z: u8 = 0x5a;

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

/// What answered, which goes in the log.
///
/// Because a menu of orb's reads the pad itself — see above — whether a pad works on one is a
/// question about orb rather than about the pad, and a log that does not say which hand answered
/// cannot settle it. It could not, and that cost a session.
#[derive(Clone, Copy, Debug)]
pub enum By {
    Keyboard,
    Pad,
}

impl fmt::Display for By {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Keyboard => "keyboard",
            Self::Pad => "pad",
        })
    }
}

/// A frame's presses. The two that can answer carry which hand made them, since naming that hand
/// is the whole of what [`By`] is for — and a cancel is an answer on one of these menus.
pub struct Pressed {
    pub up: bool,
    pub down: bool,
    pub decide: Option<By>,
    pub cancel: Option<By>,
}

/// The keys a menu of orb's own reads for itself, and the frames it holds them off for.
pub struct Keys {
    grace: u32,
    /// What the pad was doing last frame. What a menu acts on is a press and not the holding, and
    /// arriving with a button already down is the ordinary case: the key that chose the item this
    /// is asked over, or the shot key that was held when the player died.
    pad: Pad,
}

impl Keys {
    pub fn new(grace: u32) -> Self {
        Self {
            grace,
            pad: Pad::default(),
        }
    }

    /// This frame's presses, or `None` while the keys that opened the menu are still held off.
    ///
    /// The pad is read every frame, grace or not: a button held from before the menu opened must
    /// not become a press the moment the grace ends.
    pub fn read(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<Pressed> {
        let was = std::mem::replace(&mut self.pad, pad);
        let pushed = |now: bool, before: bool| now && !before;
        let pressed = Pressed {
            up: keyboard.pressed(VK_UP) || pushed(pad.up, was.up),
            down: keyboard.pressed(VK_DOWN) || pushed(pad.down, was.down),
            decide: hand(
                keyboard.pressed(VK_Z) || keyboard.pressed(VK_RETURN),
                pushed(pad.decide, was.decide),
            ),
            // The game's own cancel — `x` is its bomb key and its menus read that as back —
            // escape, which is what anything with a window on it is expected to close on, and
            // whichever pad button the game maps to that.
            cancel: hand(
                keyboard.pressed(VK_X) || keyboard.pressed(VK_ESCAPE),
                pushed(pad.cancel, was.cancel),
            ),
        };
        if self.grace > 0 {
            self.grace -= 1;
            return None;
        }
        Some(pressed)
    }

    /// Holds the next `frames` frames' keys off, for a question a press has just put up: that
    /// press is an edge and so already spent, but a question answered on the frame it appeared is
    /// a question nobody read.
    pub fn hold(&mut self, frames: u32) {
        self.grace = frames;
    }

    /// The frames left of that. For the tests, which is where holding the keys off at all is
    /// something to be said out loud rather than something a menu does.
    #[cfg(test)]
    pub fn held(&self) -> u32 {
        self.grace
    }
}

/// Which hand made a press, the keyboard where both did: the two are one press as far as anything
/// here is concerned, and one of them has to be the one named.
fn hand(keyboard: bool, pad: bool) -> Option<By> {
    match (keyboard, pad) {
        (true, _) => Some(By::Keyboard),
        (false, true) => Some(By::Pad),
        (false, false) => None,
    }
}

/// Where a cursor over `count` items goes. Wrapping, because with two items either direction is
/// the other one — a cursor that only moved one way would leave `up` doing nothing — and with
/// three the far end is one press the wrong way rather than two the right way.
pub fn moved(selection: usize, count: usize, pressed: &Pressed) -> usize {
    let mut at = selection;
    if pressed.up {
        at = at.checked_sub(1).unwrap_or(count - 1);
    }
    if pressed.down {
        at = (at + 1) % count;
    }
    at
}

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
