//! The keys of `orb.yaml`, as serde reads them, and the file as orb writes it back.
//!
//! Its own struct rather than deserialising straight into [`Config`](crate::Config), which is
//! not the same set: every option in [`args`](crate::args) is a field of `Config` and none of
//! them is a key here.
//!
//! `deny_unknown_fields` is what makes a key nobody reads an error naming it, since a setting
//! that is quietly passed over is a setting somebody thinks is on.
//!
//! Written as well as read, because these are what the launcher asks about before it starts
//! the game. Written out as text with its comments rather than through a serialiser, which
//! would leave seven bare keys and nothing beside them to say what any of it is for.

use serde::Deserialize;

use crate::Screen;

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct File {
    pub screen: Screen,
    pub always_draw: bool,
    pub boundary_flash: bool,
    pub skip_ending: bool,
    pub hide_mouse: bool,
    pub dpad_moves: bool,
    pub ask_at_startup: bool,
}

/// What each key is when the file does not say. Written out rather than derived, because
/// `bool::default()` is off and every one of these is on.
impl Default for File {
    fn default() -> Self {
        Self {
            screen: Screen::Fullscreen,
            always_draw: true,
            boundary_flash: true,
            skip_ending: true,
            hide_mouse: true,
            dpad_moves: true,
            ask_at_startup: true,
        }
    }
}

/// The file as orb writes it: every key with the comment that says what it is for, and the
/// values it was given.
pub(crate) fn text(file: &File) -> String {
    format!(
        "\
# Written by orb.exe from what it asked for before the game started, and read back by both
# halves of orb from their own directory. Deleting it is not a fault: without it every value
# below is its default and orb asks again at the next launch.
#
# A switch is written true or false, and a key nobody here reads is refused by name.
#
# What is here is what somebody playing sets and leaves set. Where the game is, building the
# midstage chapter table and looking into a fault are arguments to orb.exe instead —
# `orb --help` — because those are one machine's or one launch's and not settings.

# fullscreen, or the size of a window written WIDTHxHEIGHT. Fullscreen is borderless and covers
# the monitor. Either way the game keeps the 4:3 its 640x480 output has and the rest is black,
# which is where orb writes its own numbers. The game's own fullscreen setting is overruled, so
# what custom.exe says about it does not matter.
#
# Anything outside the game that squares windows up — vpatch's [Window] section, a borderless
# or aspect-ratio tool, a tiling window manager — wins over this, because it acts on the window
# after orb has created it. It also takes the black beside the game with it, and with that the
# numbers orb writes there. Leave the game out of such a tool while orb is doing the job.
screen: {screen}

# Never show the ending. It is run out inside the frame it starts on rather than
# jumped over, so everything it does on the way — the clear flag, the score entry —
# still happens. The staff roll after it is kept and plays as it always did. Set to
# false to watch the ending too.
skip_ending: {skip_ending}

# Keep drawing while the window is not the one in use. The game stops reading the
# keyboard then, because it reads it globally rather than per window and would
# otherwise act on whatever is being typed somewhere else.
always_draw: {always_draw}

# Wash the play field green the moment a chapter begins, which is where dying in
# pointdevice mode sends you back to. Dim and gone inside a sixth of a second; false for a
# screen with nothing of orb's on it at all. A --judge pass flashes either way, brighter
# and longer, since there the wash is what the pass is run with.
boundary_flash: {boundary_flash}

# Take the mouse pointer off the screen once the mouse has been still for three seconds, and put it
# back the moment it moves. Nothing in the game is played with the mouse and orb always puts the game
# in a window — which is where Windows draws a pointer — so without this the arrow sits over the play
# field for as long as nobody moves it away. false leaves the pointer to the game, which draws it
# whenever it moves.
hide_mouse: {hide_mouse}

# Move the player with a gamepad's d-pad as well as with its stick. The game itself reads only the
# stick, so the d-pad does nothing in it at all — while moving through the window this file was
# written from and through the questions orb asks inside the game, which both read one. So orb adds
# the direction the d-pad is pushed to the buttons the game read. false leaves the pad as the game
# has it.
dpad_moves: {dpad_moves}

# Ask for all of the above again at the next launch. false keeps what is written here and
# starts the game straight away; `orb --settings` asks whichever way this is set.
ask_at_startup: {ask_at_startup}
",
        screen = file.screen,
        skip_ending = file.skip_ending,
        always_draw = file.always_draw,
        boundary_flash = file.boundary_flash,
        hide_mouse = file.hide_mouse,
        dpad_moves = file.dpad_moves,
        ask_at_startup = file.ask_at_startup,
    )
}
