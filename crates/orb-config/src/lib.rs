//! `orb.yaml` and the command line, shared by the launcher and the injected DLL.
//!
//! The file holds what somebody playing sets and leaves set. What belongs to building the
//! midstage table or to looking into a fault is an argument instead — see [`args`] — because
//! those change from one launch to the next, and a file is the wrong place for something that
//! is different every time it is run.
//!
//! Both sides read the whole of both, so each must know every key and every option:
//! `reject_unknown_keys` would otherwise fire on the other side's.

use std::fmt;
use std::path::{Path, PathBuf};

pub mod args;
pub mod keys;
mod yaml;

pub use keys::VirtualKey;

pub const FILE_NAME: &str = "orb.yaml";

/// Keys this file used to hold, and what asks for the same thing now. A file written when
/// they were keys still parses everywhere else, so the one that moved is worth naming rather
/// than left to read as a typo.
const MOVED_TO_ARGUMENTS: &[(&str, &str)] = &[
    // The keys pressed while a table is being built are fixed in `orb`: whoever is building
    // one is the only person who ever presses them, and a setting nobody changes is a setting
    // to be rid of.
    ("save_state_key", "not a setting: the keys are fixed in the code"),
    ("load_state_key", "not a setting: the keys are fixed in the code"),
    ("tuning_add_key", "not a setting: the keys are fixed in the code"),
    ("tuning_remove_key", "not a setting: the keys are fixed in the code"),
    ("tuning_write_key", "not a setting: the keys are fixed in the code"),
    ("chapter_next_key", "not a setting: the keys are fixed in the code"),
    ("chapter_prev_key", "not a setting: the keys are fixed in the code"),
    ("chapter_hold_key", "not a setting: the keys are fixed in the code"),
    ("chapter_keep_key", "not a setting: the keys are fixed in the code"),
    ("chapter_drop_key", "not a setting: the keys are fixed in the code"),
    ("chapter_across_key", "not a setting: the keys are fixed in the code"),
    ("chapter_dropped_key", "not a setting: the keys are fixed in the code"),
    ("chapter_tuning", "--tune, or --collect and --judge for the two passes"),
    ("chapter_stepping", "--collect for the pass nobody watches, --judge for the other"),
    ("during_replay", "--replay"),
    ("replay_speed", "--speed=N"),
    ("stress_restore_frames", "--stress=N"),
    ("self_check", "--self-check"),
    ("chapters", "--no-chapters"),
    ("track_memory", "--no-memory"),
    ("frame_hooks", "--no-hooks"),
    ("log_level", "--log=quiet, --log=normal or --log=verbose"),
];

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
    /// Which kind of tuning pass this is. Off is the one that only collects: the
    /// replay runs to the end of the run and nothing stops it. On is the one somebody
    /// is watching: the game is held on each boundary, and a stage's end holds the run
    /// rather than letting it carry on into the next stage.
    pub chapter_stepping: bool,
    pub block_replay_save: bool,
    /// Keep the scores of runs orb could rewind in `orb_score.dat` and leave the game's
    /// `score.dat` alone. Off ranks them in the game's own file, which is where a run nobody
    /// could have played does not belong.
    pub own_score_file: bool,
    /// Wash the play field the moment a chapter begins, so that where dying sends you back
    /// to is something seen rather than a number to read. A judging pass flashes whichever
    /// way this is set: there the wash is what the pass is run with.
    pub boundary_flash: bool,
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
    /// Loads `orb.yaml` from the directory holding `module_path`: the launcher's own exe,
    /// or the game's when the injected DLL is asking.
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
        // Said properly rather than left to read as a typo: these were keys here once, and a
        // file kept from then is the likeliest way to meet one.
        for (key, option) in MOVED_TO_ARGUMENTS {
            if doc.string(key)?.is_some() {
                return Err(yaml::Error {
                    line: None,
                    message: format!("`{key}` is a command-line option now: {option}"),
                });
            }
        }
        let config = Self {
            game_dir: path_of("game_dir")?.unwrap_or_else(|| base_dir.clone()),
            orb_dll: path_of("orb_dll")?,
            chapters: true,
            track_memory: true,
            frame_hooks: true,
            own_frame_loop: doc.bool("own_frame_loop")?.unwrap_or(true),
            always_draw: doc.bool("always_draw")?.unwrap_or(true),
            self_check: false,
            chapter_tuning: false,
            during_replay: false,
            stress_restore_frames: 0,
            replay_speed: 1,
            chapter_stepping: false,
            block_replay_save: doc.bool("block_replay_save")?.unwrap_or(true),
            own_score_file: doc.bool("own_score_file")?.unwrap_or(true),
            boundary_flash: doc.bool("boundary_flash")?.unwrap_or(true),
            skip_ending: doc.bool("skip_ending")?.unwrap_or(true),
            borderless: doc.bool("borderless")?.unwrap_or(true),
            joystick: doc.bool("joystick")?.unwrap_or(true),
            log_level: LogLevel::Normal,
            base_dir,
        };
        doc.reject_unknown_keys()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, LogLevel};
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
        assert!(!config.chapter_stepping);
        assert!(config.block_replay_save);
        assert!(config.own_score_file);
        assert!(config.boundary_flash);
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
    /// A key that moved to the command line says so, because a file written when it was a
    /// key here is the likeliest way to meet one and `unknown key` would read as a typo.
    fn a_key_that_moved_says_where_it_went() {
        for (key, option) in super::MOVED_TO_ARGUMENTS {
            let text = format!("{key}: something\n");
            let error = match Config::parse(Path::new("orb.yaml"), &text) {
                Err(error) => error,
                Ok(_) => panic!("{key} was read from the file"),
            };
            assert!(error.message.contains(option), "{key}: {error}");
        }
    }

    #[test]
    fn log_levels_are_ordered() {
        assert_eq!(parse("").log_level, LogLevel::Normal);
        // What the macros lean on: each level carries everything below it.
        assert!(LogLevel::Verbose > LogLevel::Normal);
        assert!(LogLevel::Normal > LogLevel::Quiet);
    }
}
