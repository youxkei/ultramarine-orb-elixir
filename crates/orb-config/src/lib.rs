//! `orb.yaml`, shared by the launcher and the injected DLL.
//!
//! Both read every key from the one file, so each side must know the whole key
//! set — otherwise `reject_unknown_keys` would fire on the other side's keys.

use std::fmt;
use std::path::{Path, PathBuf};

pub mod keys;
mod yaml;

pub use keys::VirtualKey;

pub const FILE_NAME: &str = "orb.yaml";

pub struct Config {
    /// Directory holding `orb.yaml`; every relative path below resolves here.
    pub base_dir: PathBuf,
    pub game_dir: PathBuf,
    /// An `orb.dll` to load instead of the one the launcher carries inside itself. Blank
    /// for the carried one, which is what makes the launcher the only file to install.
    pub orb_dll: Option<PathBuf>,
    /// Chapters, snapshots and the retry menu. Off leaves orb loaded but with
    /// nothing of its own happening, which is what makes a fault bisectable.
    pub chapters: bool,
    /// Hook the exe's heap and reservation calls, which is what lets a snapshot
    /// cover more than `.data`.
    pub track_memory: bool,
    /// Hook the game's per-frame update and draw at all.
    pub frame_hooks: bool,
    /// Run the frame ourselves: update before draw, so nothing on screen is an
    /// update behind the input that shaped it, and paced to the display.
    pub own_frame_loop: bool,
    /// Keep drawing while the window is not the one in use.
    pub always_draw: bool,
    pub self_check: bool,
    pub chapter_tuning: bool,
    /// Track chapters while a replay plays back, so a replay can drive the work
    /// that would otherwise mean playing the whole game by hand.
    pub during_replay: bool,
    /// Restore the current chapter every this many frames, 0 to never. Exercises
    /// the snapshot and the music over and over without anyone playing.
    pub stress_restore_frames: u32,
    /// Updates to run per drawn frame while a replay plays, so a pass over a full
    /// run takes minutes rather than half an hour.
    pub replay_speed: u32,
    /// Manual save/restore, for exercising the snapshot engine by hand.
    pub save_state_key: VirtualKey,
    pub load_state_key: VirtualKey,
    /// Only read while `chapter_tuning` is on.
    pub tuning_add_key: VirtualKey,
    pub tuning_remove_key: VirtualKey,
    pub tuning_write_key: VirtualKey,
    pub block_replay_save: bool,
    /// Run the ending out without ever drawing it.
    pub skip_ending: bool,
    /// Borderless, scaled to the monitor with the game's aspect ratio kept and
    /// the rest of the screen black.
    pub borderless: bool,
    /// How much goes into `orb.log`.
    pub log_level: LogLevel,
    /// Let the game read a joystick. Off drops that read, which on a machine where
    /// the device does not answer is most of a frame's budget.
    pub joystick: bool,
}

/// How much detail the log carries.
///
/// Three tiers rather than a switch per thing worth logging, because what decides
/// them is one question: whether the log is being read to see that a run happened,
/// to see how it went, or to find out why one frame in a hundred was late.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LogLevel {
    /// What happened at startup, and anything that went wrong. What is left of a
    /// long session when nobody is looking into anything.
    Quiet,
    /// The above, plus a line a second on how the frames and the sound are doing,
    /// and a line per scene the game moves to.
    Normal,
    /// The above, plus per-frame detail: every frame the display did not get on the
    /// cadence, with where its time went.
    Verbose,
}

impl LogLevel {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "quiet" => Some(Self::Quiet),
            "normal" => Some(Self::Normal),
            "verbose" => Some(Self::Verbose),
            _ => None,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Quiet => "quiet",
            Self::Normal => "normal",
            Self::Verbose => "verbose",
        })
    }
}

#[derive(Debug)]
pub enum Error {
    Read { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, source: yaml::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Parse { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for Error {}

impl Config {
    /// Loads `orb.yaml` from the directory holding `module_path`, which is the
    /// launcher exe or `orb.dll` depending on the caller.
    pub fn load_beside(module_path: &Path) -> Result<Self, Error> {
        let base_dir = module_path.parent().unwrap_or(Path::new(".")).to_owned();
        Self::load(&base_dir.join(FILE_NAME))
    }

    pub fn load(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)
            .map_err(|source| Error::Read { path: path.to_owned(), source })?;
        Self::parse(path, &text).map_err(|source| Error::Parse { path: path.to_owned(), source })
    }

    fn parse(path: &Path, text: &str) -> Result<Self, yaml::Error> {
        let base_dir = path.parent().unwrap_or(Path::new(".")).to_owned();
        let doc = yaml::Document::parse(text)?;

        let path_of = |key| -> Result<Option<PathBuf>, yaml::Error> {
            Ok(doc.string(key)?.filter(|value| !value.is_empty()).map(|value| base_dir.join(value)))
        };
        let key_of = |key, default| -> Result<VirtualKey, yaml::Error> {
            match doc.string(key)?.filter(|value| !value.is_empty()) {
                Some(name) => keys::parse(name).ok_or_else(|| yaml::Error {
                    line: None,
                    message: format!("`{key}`: unknown key name `{name}`"),
                }),
                None => Ok(default),
            }
        };
        let config = Self {
            game_dir: path_of("game_dir")?.unwrap_or_else(|| base_dir.clone()),
            orb_dll: path_of("orb_dll")?,
            chapters: doc.bool("chapters")?.unwrap_or(true),
            track_memory: doc.bool("track_memory")?.unwrap_or(true),
            frame_hooks: doc.bool("frame_hooks")?.unwrap_or(true),
            own_frame_loop: doc.bool("own_frame_loop")?.unwrap_or(true),
            always_draw: doc.bool("always_draw")?.unwrap_or(true),
            self_check: doc.bool("self_check")?.unwrap_or(false),
            chapter_tuning: doc.bool("chapter_tuning")?.unwrap_or(false),
            during_replay: doc.bool("during_replay")?.unwrap_or(false),
            stress_restore_frames: doc.u32("stress_restore_frames")?.unwrap_or(0),
            replay_speed: doc.u32("replay_speed")?.unwrap_or(1).max(1),
            save_state_key: key_of("save_state_key", keys::C)?,
            load_state_key: key_of("load_state_key", keys::V)?,
            tuning_add_key: key_of("tuning_add_key", keys::A)?,
            tuning_remove_key: key_of("tuning_remove_key", keys::S)?,
            tuning_write_key: key_of("tuning_write_key", keys::D)?,
            block_replay_save: doc.bool("block_replay_save")?.unwrap_or(true),
            skip_ending: doc.bool("skip_ending")?.unwrap_or(true),
            borderless: doc.bool("borderless")?.unwrap_or(true),
            joystick: doc.bool("joystick")?.unwrap_or(true),
            log_level: match doc.string("log_level")?.filter(|value| !value.is_empty()) {
                Some(name) => LogLevel::parse(&name).ok_or_else(|| yaml::Error {
                    line: None,
                    message: format!(
                        "`log_level`: `{name}` is not one of quiet, normal or verbose"
                    ),
                })?,
                None => LogLevel::Normal,
            },
            base_dir,
        };
        doc.reject_unknown_keys()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, LogLevel, keys};
    use std::path::{Path, PathBuf};

    fn parse(text: &str) -> Config {
        Config::parse(Path::new("/opt/orb/orb.yaml"), text).unwrap()
    }

    #[test]
    fn defaults_place_everything_beside_the_config() {
        let config = parse("");
        assert_eq!(config.game_dir, PathBuf::from("/opt/orb"));
        // The launcher carries orb.dll; a path here is an override, not the normal case.
        assert_eq!(config.orb_dll, None);
        assert_eq!(config.save_state_key, keys::C);
        assert_eq!(config.load_state_key, keys::V);
        assert_eq!(config.tuning_add_key, keys::A);
        assert_eq!(config.tuning_remove_key, keys::S);
        assert_eq!(config.tuning_write_key, keys::D);
        assert!(config.block_replay_save);
        assert!(config.skip_ending);
        assert!(config.borderless);
        assert!(config.chapters);
        assert!(config.track_memory);
        assert!(config.frame_hooks);
        assert!(config.own_frame_loop);
        assert!(config.always_draw);
        assert!(!config.self_check);
        assert!(!config.chapter_tuning);
        assert!(!config.during_replay);
        assert_eq!(config.stress_restore_frames, 0);
        assert_eq!(config.replay_speed, 1);
    }

    #[test]
    fn relative_paths_resolve_against_the_config_directory() {
        let config = parse("game_dir: game\n");
        assert_eq!(config.game_dir, PathBuf::from("/opt/orb/game"));
    }

    #[test]
    fn absolute_paths_are_kept_as_written() {
        let config = parse("game_dir: /srv/th06\n");
        assert_eq!(config.game_dir, PathBuf::from("/srv/th06"));
    }

    #[test]
    fn key_names_are_checked() {
        assert_eq!(parse("save_state_key: shift\n").save_state_key, keys::SHIFT);
        assert!(Config::parse(Path::new("orb.yaml"), "save_state_key: nonesuch\n").is_err());
    }

    #[test]
    fn log_levels_are_named_and_ordered() {
        assert_eq!(parse("").log_level, LogLevel::Normal);
        assert_eq!(parse("log_level: quiet\n").log_level, LogLevel::Quiet);
        assert_eq!(parse("log_level: verbose\n").log_level, LogLevel::Verbose);
        // What the macros lean on: each level carries everything below it.
        assert!(LogLevel::Verbose > LogLevel::Normal);
        assert!(LogLevel::Normal > LogLevel::Quiet);
        assert!(Config::parse(Path::new("orb.yaml"), "log_level: loud\n").is_err());
    }
}
