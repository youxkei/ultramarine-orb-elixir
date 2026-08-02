//! The keys of `orb.yaml`, as serde reads them.
//!
//! Its own struct rather than deserialising straight into [`Config`](crate::Config), which is
//! not the same set: every option in [`args`](crate::args) is a field of `Config` and none of
//! them is a key here, and `game_dir` is a path resolved against the file's own directory
//! rather than the string that is written down.
//!
//! `deny_unknown_fields` is what makes a key nobody reads an error naming it, since a setting
//! that is quietly passed over is a setting somebody thinks is on. It is also what rejects a
//! file still carrying `joystick`, which was a key until the read it turned off moved to a
//! thread of orb's own.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct File {
    /// Blank, which is `None` here, for the directory the file itself is in.
    pub game_dir: Option<String>,
    pub orb_dll: Option<String>,
    pub own_frame_loop: bool,
    pub always_draw: bool,
    pub block_replay_save: bool,
    pub own_score_file: bool,
    pub boundary_flash: bool,
    pub skip_ending: bool,
    pub borderless: bool,
}

/// What each key is when the file does not say. Written out rather than derived, because
/// `bool::default()` is off and every one of these is on.
impl Default for File {
    fn default() -> Self {
        Self {
            game_dir: None,
            orb_dll: None,
            own_frame_loop: true,
            always_draw: true,
            block_replay_save: true,
            own_score_file: true,
            boundary_flash: true,
            skip_ending: true,
            borderless: true,
        }
    }
}
