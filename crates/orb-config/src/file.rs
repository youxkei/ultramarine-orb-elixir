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

/// The word `language` and `thcrap` take for the machine's own, which is neither the name of a language
/// nor a switch: it says where the answer comes from, and by the time a screen is drawn or a game is
/// started it has been answered.
const AUTO: &str = "auto";

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct File {
    pub screen: Screen,
    #[serde(deserialize_with = "language")]
    pub language: Option<Language>,
    #[serde(deserialize_with = "switch_or_auto")]
    pub thcrap: Option<bool>,
    pub always_draw: bool,
    pub boundary_flash: bool,
    pub skip_ending: bool,
    pub hide_mouse: bool,
    pub dpad_moves: bool,
    pub save_replays: bool,
    pub player_name: String,
    pub ask_at_startup: bool,
}

/// What each key is when the file does not say. Written out rather than derived, because
/// `bool::default()` is off and every switch here but one is on.
///
/// **That one is `save_replays`**, which is off: a run's replay is offered rather than kept, because
/// keeping every one of them fills a directory nobody asked to have filled, and a question at the end
/// of a run is where somebody says which are worth keeping. The name beside it is empty for a reason
/// of the same kind — nobody has said whose runs these are, and orb inventing one would put a name
/// somebody did not choose into every replay they save.
impl Default for File {
    fn default() -> Self {
        Self {
            screen: Screen::Fullscreen,
            language: None,
            thcrap: None,
            always_draw: true,
            boundary_flash: true,
            skip_ending: true,
            hide_mouse: true,
            dpad_moves: true,
            save_replays: false,
            player_name: String::new(),
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

/// A switch that has [`AUTO`] as well as the two words every other switch is written with, the answer
/// then coming from the same place the language's does.
///
/// Read as a string like the language above, YAML's own `true` and `false` included: a switch and a
/// word cannot both arrive as one type any other way, and the three words this accepts are the three a
/// reader of the file sees.
fn switch_or_auto<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<bool>, D::Error> {
    let text = String::deserialize(deserializer)?;
    let text = text.trim();
    if text.eq_ignore_ascii_case(AUTO) {
        return Ok(None);
    }
    match text.parse::<bool>() {
        Ok(switch) => Ok(Some(switch)),
        Err(_) => Err(serde::de::Error::custom(format!(
            "{text:?} is not true, false or {AUTO}"
        ))),
    }
}

/// A name as a YAML scalar nothing can misread: double-quoted, with the two characters that mode
/// gives a meaning of their own put back as escapes.
///
/// Quoted rather than written bare because this is the one key whose value is somebody's own text:
/// `12` bare is a number, `yes` is a switch, a leading `#` starts a comment and a colon inside it ends
/// the key. Double quotes rather than single because a name is Japanese as often as not and the escapes
/// are two characters either way.
fn quoted(name: &str) -> String {
    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
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
save_replays: {save_replays}
player_name: {player_name}
ask_at_startup: {ask_at_startup}
",
        screen = file.screen,
        thcrap = match file.thcrap {
            None => AUTO.to_owned(),
            Some(switch) => switch.to_string(),
        },
        language = match file.language {
            None => AUTO.to_owned(),
            Some(language) => language.to_string(),
        },
        skip_ending = file.skip_ending,
        always_draw = file.always_draw,
        boundary_flash = file.boundary_flash,
        hide_mouse = file.hide_mouse,
        dpad_moves = file.dpad_moves,
        save_replays = file.save_replays,
        player_name = quoted(&file.player_name),
        ask_at_startup = file.ask_at_startup,
    )
}
