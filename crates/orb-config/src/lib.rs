//! `orb.yaml` and the command line, shared by the launcher and the injected DLL.
//!
//! The file holds what somebody playing sets and leaves set. What belongs to building the
//! midstage table or to looking into a fault is an argument instead — see [`args`] — because
//! those change from one launch to the next, and a file is the wrong place for something that
//! is different every time it is run.
//!
//! Both sides read the whole of both, so each must know every key and every option:
//! `deny_unknown_fields` and clap would otherwise each refuse the other side's.

use std::fmt;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

pub mod args;
mod file;
pub mod keys;

pub use keys::VirtualKey;

pub const FILE_NAME: &str = "orb.yaml";

pub struct Config {
    /// Directory `orb.yaml` is looked for in, which for the DLL is also what it writes its log
    /// and its tuning table beside.
    pub base_dir: PathBuf,
    /// Directory holding 東方紅魔郷.exe.
    ///
    /// The one `base_dir` is, which for the injected DLL is where it already is. The launcher
    /// is the only side that can be somewhere else, and `--game-dir` is how it is told.
    pub game_dir: PathBuf,
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
    ///
    /// Off through `--no-frame-loop` and nothing else. A run without it is a run with the
    /// frame of input lag and the doubled frames back, which is a thing to do to a fault
    /// and not a way to leave the game set up.
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
    /// Updates to run per drawn frame while a replay plays or a run is being cleared
    /// with `fast_clear`, so a pass or a clear over a full run takes a minute rather
    /// than half an hour.
    pub speed: u32,
    /// Nothing can hit the player, and a run someone is playing runs at `speed`.
    ///
    /// For reaching the ending, which nothing else can reach: the game enters one only
    /// after a run somebody played cleared it, so watching what happens there means
    /// clearing it, and a clear is half an hour of playing well.
    pub fast_clear: bool,
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
    /// Run the ending out without ever drawing it, stopping at its staff roll.
    pub skip_ending: bool,
    /// Borderless, scaled to the monitor with the game's aspect ratio kept and
    /// the rest of the screen black.
    pub borderless: bool,
    /// How much goes into `orb.log`.
    pub log_level: LogLevel,
    /// Write what every frame that missed the cadence spent its turn on, whatever
    /// `log_level` says.
    ///
    /// Its own switch rather than a tier of the level, because a frame is late for
    /// reasons that include what the log itself was writing at the time: `verbose` has
    /// orb's joystick thread writing twice a second and the tuning pass writing per
    /// boundary, and those writes serialise against the frame's own. Asking for the
    /// pacing at `quiet` leaves nothing in the log but this, which is the only way to
    /// watch the pacing without the watching being one of the causes.
    pub pacing_log: bool,
    /// How long the compositor is left to draw in, between a frame being handed over and
    /// the blank it is to be shown at, in microseconds. 0 to find it while running.
    ///
    /// That window is not idle: it is where the desktop is composed and got onto the screen
    /// for that blank, so a frame's turn holds two drawing times and both have to finish
    /// before the blank. The compositor does not say how long it wants, and the count it
    /// offers for judging it — `cFramesLate` — stays at zero through runs whose cadence is
    /// visibly broken. So it is swept instead: pinned small enough that frames are known to
    /// miss their blank, then walked up until they stop. Pinning also takes the adjustment
    /// out of the way of anything else being measured.
    pub compose_us: u32,
    // There is no `joystick` here on purpose. It existed to turn off a read that cost most of
    // a frame; that read is on a thread of orb's own now and costs the frame nothing, so
    // there is nothing left to turn off. A file still carrying the key is rejected by name,
    // which says what to delete better than quietly ignoring it would.
}

/// How much detail the log carries.
///
/// Three tiers rather than a switch per thing worth logging, because what decides
/// them is one question: whether the log is being read to see that a run happened,
/// to see how it went, or to find out why one frame in a hundred was late.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
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
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_yaml_ng::Error,
    },
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
    /// Loads `orb.yaml` from the directory holding `module_path`: the launcher's own exe, or
    /// the game's when the injected DLL is asking.
    ///
    /// No file there is every default, since every key is one thing somebody changed and
    /// changing nothing is what most installations do. Installing orb is then the one exe.
    pub fn load_beside(module_path: &Path) -> Result<Self, Error> {
        let base_dir = module_path.parent().unwrap_or(Path::new(".")).to_owned();
        let path = base_dir.join(FILE_NAME);
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(&path, &text),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::from_file(base_dir, file::File::default()))
            }
            Err(source) => Err(Error::Read { path, source }),
        }
    }

    /// Loads a named `orb.yaml`, which is what `--config` gives.
    ///
    /// Not there is an error here, unlike the file orb looks for itself: a path somebody typed
    /// is a path they meant, and reading the defaults instead would leave them watching for a
    /// setting that was never read.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path).map_err(|source| Error::Read {
            path: path.to_owned(),
            source,
        })?;
        Self::parse(path, &text)
    }

    fn parse(path: &Path, text: &str) -> Result<Self, Error> {
        // Through `Option` rather than straight into `File`, because a file with nothing in it
        // but comments is an empty document — `null`, which is not a mapping — and every key
        // having a default is the whole point of it being allowed.
        let file: Option<file::File> =
            serde_yaml_ng::from_str(text).map_err(|source| Error::Parse {
                path: path.to_owned(),
                source,
            })?;
        let base_dir = path.parent().unwrap_or(Path::new(".")).to_owned();
        Ok(Self::from_file(base_dir, file.unwrap_or_default()))
    }

    fn from_file(base_dir: PathBuf, file: file::File) -> Self {
        Self {
            game_dir: base_dir.clone(),
            chapters: true,
            track_memory: true,
            frame_hooks: true,
            own_frame_loop: true,
            always_draw: file.always_draw,
            self_check: false,
            chapter_tuning: false,
            during_replay: false,
            stress_restore_frames: 0,
            speed: 1,
            fast_clear: false,
            chapter_stepping: false,
            block_replay_save: file.block_replay_save,
            own_score_file: file.own_score_file,
            boundary_flash: file.boundary_flash,
            skip_ending: file.skip_ending,
            borderless: file.borderless,
            log_level: LogLevel::Normal,
            pacing_log: false,
            compose_us: 0,
            base_dir,
        }
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
        // Nothing in the file says where the game is: it is where the file is, and the launcher
        // is the only side that can be told otherwise.
        assert_eq!(config.game_dir, PathBuf::from("/opt/orb"));
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
        assert_eq!(config.speed, 1);
    }

    /// No file at all is every default, which is what leaves the launcher the one file to
    /// install: it hands `load_beside` its own path, and the directory that is in is where the
    /// game, the log and the scores are.
    #[test]
    fn no_file_beside_the_exe_is_every_default() {
        let dir = std::env::temp_dir().join(format!("orb-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(
            !dir.join(super::FILE_NAME).exists(),
            "left over in {}",
            dir.display()
        );

        let config = Config::load_beside(&dir.join("orb-launcher.exe")).unwrap();
        assert_eq!(config.game_dir, dir);
        assert!(config.borderless);
        assert!(config.skip_ending);
        assert!(config.always_draw);
        assert!(config.block_replay_save);
        assert!(config.own_score_file);
        assert!(config.boundary_flash);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// A file `--config` names and does not find is an error, where the one orb looks for
    /// itself is not: reading the defaults instead would leave somebody watching for a setting
    /// nothing ever read.
    #[test]
    fn a_named_file_that_is_not_there_says_so() {
        let missing = std::env::temp_dir().join("orb-config-no-such-file.yaml");
        let error = match Config::load(&missing) {
            Err(error) => error.to_string(),
            Ok(_) => panic!("a file that is not there was read as the defaults"),
        };
        assert!(error.contains("orb-config-no-such-file.yaml"), "{error}");
    }

    /// What the shipped file is made of: a comment per key, a blank line between them, and a
    /// comment after a value on the same line as it.
    #[test]
    fn reads_the_file_as_it_is_written() {
        let config = parse(
            "# Borderless, filling the monitor.\n\
             borderless: false  # not on this machine\n\
             \n\
             # Never show the ending.\n\
             skip_ending: false\n",
        );
        assert!(!config.borderless);
        assert!(!config.skip_ending);
        assert!(config.always_draw);
    }

    /// Anything this file does not know is an error rather than something passed over, since a
    /// setting that is quietly not read is a setting somebody thinks is on. What an option is
    /// named is not a key here, however much it reads like one.
    #[test]
    fn a_key_that_is_not_a_key_says_so() {
        for key in ["chapter_tuning", "own_frame_loop"] {
            let text = format!("{key}: true\n");
            let error = match Config::parse(Path::new("orb.yaml"), &text) {
                Err(error) => error.to_string(),
                Ok(_) => panic!("{key} was read from the file"),
            };
            assert!(error.contains(key), "{key}: {error}");
        }
    }

    /// The ways a file written by hand goes wrong: a switch that is not one, a key with
    /// nothing after it to say `key: value`, and a key written twice.
    #[test]
    fn what_cannot_be_read_says_so() {
        for (text, complaint) in [
            ("borderless: maybe\n", "expected a boolean"),
            ("borderless\n", "invalid type"),
            ("borderless: true\nborderless: false\n", "duplicate"),
        ] {
            let error = match Config::parse(Path::new("orb.yaml"), text) {
                Err(error) => error.to_string(),
                Ok(_) => panic!("{text:?} was read"),
            };
            assert!(error.contains(complaint), "{text:?}: {error}");
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
