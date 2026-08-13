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
//! the game.
//!
//! **Nothing but the keys and their values is written.** What each of them is for is said by the
//! settings dialog, which asks all of them in the language the person answering reads — so a comment
//! over each key would be the one thing orb installs that is in one language whoever gets it may not
//! have. A file somebody has written comments into keeps them until the dialog writes that file back,
//! which is the same moment every value in it is replaced by what was just answered.
//!
//! Written by hand rather than through a serialiser even so: [`Screen`] is a word or a size and the
//! language is a word or `auto`, and both are written here exactly as they are read back, where a
//! serialiser would be two `Serialize` implementations saying the same thing further away from the
//! parsing they have to agree with.

use serde::Deserialize;

use crate::{Language, Screen};

/// The word `language` takes for the machine's own, which is not the name of a language: it says
/// where the answer comes from, and by the time a screen is drawn it has been answered.
const AUTO: &str = "auto";

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct File {
    pub screen: Screen,
    #[serde(deserialize_with = "language")]
    pub language: Option<Language>,
    pub thcrap: bool,
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
            language: None,
            thcrap: true,
            always_draw: true,
            boundary_flash: true,
            skip_ending: true,
            hide_mouse: true,
            dpad_moves: true,
            ask_at_startup: true,
        }
    }
}

/// One of the two languages by name, or [`AUTO`].
///
/// Through a string rather than through `Option`'s own deserialiser, which would want `null` in the
/// file for the machine's own: what is written there is one of three words.
fn language<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Language>, D::Error> {
    let text = String::deserialize(deserializer)?;
    if text.trim().eq_ignore_ascii_case(AUTO) {
        return Ok(None);
    }
    text.parse().map(Some).map_err(serde::de::Error::custom)
}

/// The file as orb writes it: every key and the value it was given.
pub(crate) fn text(file: &File) -> String {
    format!(
        "\
screen: {screen}
language: {language}
thcrap: {thcrap}
skip_ending: {skip_ending}
always_draw: {always_draw}
boundary_flash: {boundary_flash}
hide_mouse: {hide_mouse}
dpad_moves: {dpad_moves}
ask_at_startup: {ask_at_startup}
",
        screen = file.screen,
        thcrap = file.thcrap,
        language = match file.language {
            None => AUTO.to_owned(),
            Some(language) => language.to_string(),
        },
        skip_ending = file.skip_ending,
        always_draw = file.always_draw,
        boundary_flash = file.boundary_flash,
        hide_mouse = file.hide_mouse,
        dpad_moves = file.dpad_moves,
        ask_at_startup = file.ask_at_startup,
    )
}
