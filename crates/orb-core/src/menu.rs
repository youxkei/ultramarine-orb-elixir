//! The keys the questions orb puts on the screen read, and where a cursor over them goes.
//!
//! Each of those questions is up on frames the game is frozen on, which means its own input handling
//! is not running either — so each reads the keyboard itself and takes the pad from the sample orb's
//! own thread keeps. Without that second half a pad does nothing at all on one of these while
//! working perfectly on the game's own menu a keypress earlier, which is the whole of how it looks
//! broken.
//!
//! Here rather than three times over because this is where the corrections landed: the pad's edge,
//! the frames a menu holds its keys off for, which button cancels, and which hand answered were
//! each found once and had to be fixed in three places.
//!
//! Apart from the drawing — `menu_ui` in the `orb` crate — because what these decide is a function
//! of a keyboard and a pad, and both of those are things a test can hand over. The labels are not.
//!
//! **A menu of orb's has to read the pad itself.** Freezing the game stops its input read, so on those
//! frames a pad drives nothing — which looks like the pad being broken, since it worked on the game's own
//! menu one keypress earlier. So both of orb's menus ask the game what the pad is doing, and the answer is
//! read the way the game reads it: its own DirectInput controller where it has one, the winmm sample orb's
//! thread keeps where it has not, and either way through `g_Supervisor.cfg.controllerMapping` — the same
//! copy `Controller::GetControllerInput` reads. Which of the two matters more than it sounds; see
//! [`crate::joystick`].
//!
//! **Shoot decides; bomb and menu cancel.** Which is what the game's own menus do: `TH_BUTTON_SELECTMENU` is
//! `TH_BUTTON_ENTER | TH_BUTTON_SHOOT` and `TH_BUTTON_RETURNMENU` is `TH_BUTTON_MENU | TH_BUTTON_BOMB`, so
//! either of those two is a back there. The settings dialog reads it the same way, out of the game's
//! configuration file rather than the game's memory, because a button that answers a question before the
//! game starts and cancels one inside it is worse than either.
//!
//! The menu button decided for a while instead. On the pad orb was first run with it was button 0 — where a
//! thumb rests — so the most obvious button on the pad closed the question instead of answering it. That is
//! what the launcher printing the mapping it read is for, that being the only place it is written down in a
//! form anybody can look at: a pad where shoot and menu are the other way round is a pad on which orb's
//! menus will look like they cancel themselves, and the printed line is where to see it.
//!
//! Up and down come from that mapping too, and from the Y axis read the way that function reads it — the
//! centre halfway between `g_JoyCaps`' bounds and a dead zone of a quarter of the travel either side, its
//! low side being up since the axis is measured downwards — and from the hat, which is where a d-pad reports
//! and which the game itself does not read at all. All of it on the press rather than the holding, and the
//! previous frame's state is kept through the grace frames as well — a button held from before the menu
//! opened must not become a press the moment the grace ends.

use std::fmt;

use crate::game::Pad;
use crate::input::Keyboard;

const VK_RETURN: u8 = 0x0d;
const VK_ESCAPE: u8 = 0x1b;
const VK_UP: u8 = 0x26;
const VK_DOWN: u8 = 0x28;
const VK_X: u8 = 0x58;
const VK_Z: u8 = 0x5a;

/// What answered, which goes in the log.
///
/// Because a menu of orb's reads the pad itself — see above — whether a pad works on one is a
/// question about orb rather than about the pad, and a log that does not say which hand answered
/// cannot settle it. It could not, and that cost a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum By {
    Keyboard,
    Pad,
    /// **And a recipe's own ask**, which is neither: what a launch playing itself presses arrives as a pad, but
    /// the one answer it cannot press for is asked for outright — see
    /// `runtime::give_the_run_up_at_the_retry`. Named apart from the two hands for the reason they are named at
    /// all: a log that said `pad` here would be a log saying a press answered where none did.
    Recipe,
}

impl fmt::Display for By {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Keyboard => "keyboard",
            Self::Pad => "pad",
            Self::Recipe => "recipe's own ask",
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
    ///
    /// **`cfg(test)` and not `feature = "sim"` as well**, which it was: an e2e test has no way to a
    /// `Keys` at all — each of the three menus keeps its own privately, and the menu a run is answering
    /// is inside orb's runtime — so what an e2e test reads instead is the log line the answer wrote.
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
