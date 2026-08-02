//! The menu that appears where the chapter was lost.
//!
//! The game is frozen while this is up, which means its own input handling is
//! not running either — so the menu reads the keyboard itself, and takes the pad
//! from the sample orb's own thread keeps.

use crate::game::{Pad, Rect};
use crate::input::Keyboard;
use crate::mode_ui::By;
use crate::overlay::{Label, Overlay};

const VK_RETURN: u8 = 0x0d;
const VK_UP: u8 = 0x26;
const VK_DOWN: u8 = 0x28;
const VK_Z: u8 = 0x5a;

const DIM: u32 = 0xb400_0000;
const NORMAL: u32 = 0xffff_ffff;
const SELECTED: u32 = 0xffff_e066;

const LINE_HEIGHT: f32 = 24.0;

/// Frames before the menu accepts anything. The player was holding keys when they
/// died — very likely a direction and the shoot key — and those presses belong to
/// the run, not to this menu.
const INPUT_GRACE_FRAMES: u32 = 24;

#[derive(Clone, Copy)]
pub enum Choice {
    Chapter,
    Stage,
}

impl Choice {
    /// For the log, in the English the rest of it is in — the menu itself is in Japanese.
    pub fn label(self) -> &'static str {
        match self {
            Self::Chapter => "the chapter again",
            Self::Stage => "the stage again",
        }
    }
}

const CHOICES: [(Choice, &str); 2] = [
    (Choice::Chapter, "チャプターをやり直す"),
    (Choice::Stage, "ステージをやり直す"),
];

pub struct RetryMenu {
    selection: usize,
    grace: u32,
    /// What the pad was doing last frame. The game is frozen here, so it is not reading the pad
    /// itself and this menu would take nothing from one at all — and dying with a pad in hand is
    /// exactly when this menu comes up.
    pad: Pad,
    chapter: Label,
    retry: Label,
    choices: [Label; CHOICES.len()],
    cursor: Label,
}

impl RetryMenu {
    pub fn new() -> Self {
        Self {
            selection: 0,
            grace: INPUT_GRACE_FRAMES,
            pad: Pad::default(),
            chapter: Label::new(),
            retry: Label::new(),
            choices: [Label::new(), Label::new()],
            cursor: Label::new(),
        }
    }

    /// Returns the choice once it is confirmed. `pad` is what the pad is doing now, read for this
    /// menu by the caller.
    ///
    /// Nothing cancels: the player is dead, and the two choices are the only two ways on.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Choice, By)> {
        // Every frame, grace or not: the player was holding the shot key when they died, and that
        // must not become a press the moment the grace ends.
        let was = std::mem::replace(&mut self.pad, pad);
        let pushed = |now: bool, before: bool| now && !before;
        if self.grace > 0 {
            self.grace -= 1;
            return None;
        }
        if keyboard.pressed(VK_UP) || pushed(pad.up, was.up) {
            self.selection = self.selection.checked_sub(1).unwrap_or(CHOICES.len() - 1);
        }
        if keyboard.pressed(VK_DOWN) || pushed(pad.down, was.down) {
            self.selection = (self.selection + 1) % CHOICES.len();
        }
        let chosen = CHOICES[self.selection].0;
        if keyboard.pressed(VK_Z) || keyboard.pressed(VK_RETURN) {
            return Some((chosen, By::Keyboard));
        }
        pushed(pad.decide, was.decide).then_some((chosen, By::Pad))
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay, area: Rect, chapter: &str, retries: u32) {
        unsafe {
            self.chapter.set(overlay, chapter);
            self.retry.set(overlay, &format!("RETRY {retries}"));
            self.cursor.set(overlay, "▶");
            for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                label.set(overlay, text);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        frame.fill(area.left, area.top, area.width, area.height, DIM);

        let center = area.center_x();
        let mut y = area.center_y() - LINE_HEIGHT * 3.0;
        for label in [&self.chapter, &self.retry] {
            frame.label(label, center - label.width() / 2.0, y, NORMAL);
            y += LINE_HEIGHT;
        }

        y += LINE_HEIGHT;
        for (index, label) in self.choices.iter().enumerate() {
            let selected = index == self.selection;
            let x = center - label.width() / 2.0;
            frame.label(label, x, y, if selected { SELECTED } else { NORMAL });
            if selected {
                frame.label(&self.cursor, x - self.cursor.width() - 6.0, y, SELECTED);
            }
            y += LINE_HEIGHT;
        }
    }
}
