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
use std::str::FromStr;

use clap::ValueEnum;
use serde::Deserialize;

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
    /// Have the game read its keyboard the way it does when DirectInput has no device, so that keys
    /// another program sends are seen.
    ///
    /// For driving a session from a script, which is otherwise impossible: the game takes its
    /// keyboard `DISCL_EXCLUSIVE | DISCL_FOREGROUND` and such a device does not see `SendInput`, so
    /// every screen — the game's menus and the questions orb puts over them — needs a hand. Measured
    /// rather than assumed, and `orb-e2e`'s `keys_from_another_program` is that measurement as an
    /// e2e test: a key another program sent reaches the game only once orb has let the device go.
    pub sent_keys: bool,
    /// Write down what a pointdevice run has pressed, so the chapter it is left in can be
    /// played again in a later launch.
    ///
    /// Not a key in `orb.yaml`. Off through `--no-resume`, which is there because this hooks the
    /// game's own input read and the moment a stage's numbers are put in place and writes a file
    /// at every chapter — so a fault in a run that chapters alone do not explain wants a way to
    /// take exactly that out. Off through `--clear` as well, which has nothing worth writing
    /// down: see there.
    pub resume: bool,
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
    /// Refuse to write replay files.
    ///
    /// Not a key in `orb.yaml` and set by `--clear` alone. A pointdevice run is never
    /// offered the screen that saves one, so there is nothing there for a switch to turn
    /// off; a cleared run is offered it, and what it would record is not what happened.
    pub block_replay_save: bool,
    /// Wash the play field the moment a chapter begins, so that where dying sends you back
    /// to is something seen rather than a number to read. A judging pass flashes whichever
    /// way this is set: there the wash is what the pass is run with.
    pub boundary_flash: bool,
    /// Run the ending out without ever drawing it, stopping at its staff roll.
    pub skip_ending: bool,
    /// Take the mouse pointer off the screen while nothing has moved the mouse for a few seconds,
    /// and put it back the moment something does.
    ///
    /// Off leaves the pointer to the game and the exe's `ShowCursor` import unpatched with it: what
    /// orb does about the pointer is own the host's display counter, and there is nothing to own where
    /// the pointer is not being taken off.
    pub hide_mouse: bool,
    /// Add the direction a pad's d-pad is pushed to the buttons the game read, so that the player
    /// moves on it as well as on the stick.
    ///
    /// The game reads a pad's two axes and neither of the two fields a d-pad reports in, so without
    /// this a d-pad does nothing in the game — while driving the launcher's own dialog and orb's own
    /// menus, both of which read it. Off hands the game the word its own read produced.
    pub dpad_moves: bool,
    /// How big the game's window is, the game's aspect ratio kept either way.
    pub screen: Screen,
    /// Ask for the settings above before starting the game, and write down what was
    /// answered. The launcher's question alone: the DLL is inside a game that has already
    /// started.
    pub ask_at_startup: bool,
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

/// How much of the screen the game gets.
///
/// One setting rather than a switch and a size, because they are answers to one question and
/// a size that is only read when the switch is off is a size somebody has set and is not
/// getting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    /// Borderless and covering the monitor, with the game's aspect ratio kept and the rest
    /// of the screen black.
    Fullscreen,
    /// A window this many pixels across inside its frame, centred on the monitor. The game
    /// keeps its own aspect ratio inside it, so a 16:9 window is the game with black down
    /// both sides — which is where orb's own numbers go.
    Window { width: u32, height: u32 },
}

impl Screen {
    /// Below this a window is not one: too small to hold anything of a 640x480 game, and small
    /// enough that a size this low in the file is a typing mistake rather than a wish. Low enough
    /// to be out of the way of anything anybody would ask for.
    const MIN: u32 = 64;
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fullscreen => f.write_str("fullscreen"),
            Self::Window { width, height } => write!(f, "{width}x{height}"),
        }
    }
}

impl FromStr for Screen {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("fullscreen") {
            return Ok(Self::Fullscreen);
        }
        // `x` rather than `*`, because that is how a resolution is written everywhere else
        // it is written, including in the list the launcher offers.
        let size = text
            .split_once(['x', 'X'])
            .and_then(|(width, height)| {
                Some((width.trim().parse().ok()?, height.trim().parse().ok()?))
            })
            .filter(|(width, height)| *width >= Self::MIN && *height >= Self::MIN);
        match size {
            Some((width, height)) => Ok(Self::Window { width, height }),
            None => Err(format!(
                "{text:?} is not fullscreen or a window size like 1280x720"
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Screen {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Through a string, since that is what YAML makes of both `fullscreen` and `1280x720`.
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
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
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Parse { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Write { path, source } => write!(f, "cannot write {}: {source}", path.display()),
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
        match orb_api::fs::read_to_string(&path) {
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
        let text = orb_api::fs::read_to_string(path).map_err(|source| Error::Read {
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
            resume: true,
            sent_keys: false,
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
            block_replay_save: false,
            boundary_flash: file.boundary_flash,
            skip_ending: file.skip_ending,
            hide_mouse: file.hide_mouse,
            dpad_moves: file.dpad_moves,
            screen: file.screen,
            ask_at_startup: file.ask_at_startup,
            log_level: LogLevel::Normal,
            pacing_log: false,
            compose_us: 0,
            base_dir,
        }
    }

    /// Writes the keys of `orb.yaml` back, which is what the launcher does with the answers to
    /// the settings it asked for.
    ///
    /// Only the keys: everything else in a `Config` came off a command line, and a launch's
    /// arguments are not somebody's settings.
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let file = file::File {
            screen: self.screen,
            always_draw: self.always_draw,
            boundary_flash: self.boundary_flash,
            skip_ending: self.skip_ending,
            hide_mouse: self.hide_mouse,
            dpad_moves: self.dpad_moves,
            ask_at_startup: self.ask_at_startup,
        };
        orb_api::fs::write(path, file::text(&file).as_bytes()).map_err(|source| Error::Write {
            path: path.to_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, LogLevel, Screen};
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
        // Not a key: only `--clear` refuses a replay, and a file that says so is a file
        // somebody has to edit back.
        assert!(!config.block_replay_save);
        assert!(config.boundary_flash);
        assert!(config.skip_ending);
        assert!(config.hide_mouse);
        assert!(config.dpad_moves);
        assert_eq!(config.screen, Screen::Fullscreen);
        assert!(config.ask_at_startup);
        assert!(config.chapters);
        assert!(config.track_memory);
        assert!(config.resume);
        assert!(!config.sent_keys);
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

        let config = Config::load_beside(&dir.join("orb.exe")).unwrap();
        assert_eq!(config.game_dir, dir);
        assert_eq!(config.screen, Screen::Fullscreen);
        assert!(config.skip_ending);
        assert!(config.always_draw);
        assert!(config.boundary_flash);
        assert!(config.hide_mouse);
        assert!(config.dpad_moves);
        assert!(config.ask_at_startup);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// What the launcher writes is what the next launch reads, since the file it writes is the
    /// file both halves of orb then read. Every key, so a key added to one side and not the
    /// other is caught here.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        let dir = std::env::temp_dir().join(format!("orb-config-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(super::FILE_NAME);

        let mut config = parse("");
        config.screen = Screen::Window {
            width: 1440,
            height: 1080,
        };
        config.always_draw = false;
        config.boundary_flash = false;
        config.skip_ending = false;
        config.hide_mouse = false;
        config.dpad_moves = false;
        config.ask_at_startup = false;
        config.save(&path).unwrap();

        let read = Config::load(&path).unwrap();
        assert_eq!(read.screen, config.screen);
        assert!(!read.always_draw);
        assert!(!read.boundary_flash);
        assert!(!read.skip_ending);
        assert!(!read.hide_mouse);
        assert!(!read.dpad_moves);
        assert!(!read.ask_at_startup);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The window size as it is written everywhere else a resolution is written, and
    /// fullscreen as a word rather than as a size nothing could mean.
    #[test]
    fn a_screen_is_fullscreen_or_a_size() {
        assert_eq!(parse("screen: fullscreen\n").screen, Screen::Fullscreen);
        assert_eq!(
            parse("screen: 1280x720\n").screen,
            Screen::Window {
                width: 1280,
                height: 720
            }
        );
        // Written back the way it is read, which is what makes the round trip above one.
        assert_eq!(Screen::Fullscreen.to_string(), "fullscreen");
        assert_eq!(
            Screen::Window {
                width: 640,
                height: 480
            }
            .to_string(),
            "640x480"
        );
    }

    #[test]
    fn a_screen_that_is_neither_says_so() {
        for text in ["window", "1280", "1280x", "x720", "1280x720x60", "8x8", ""] {
            let error = match Config::parse(Path::new("orb.yaml"), &format!("screen: {text:?}\n")) {
                Err(error) => error.to_string(),
                Ok(config) => panic!("{text:?} was read as {}", config.screen),
            };
            assert!(error.contains("1280x720"), "{text:?}: {error}");
        }
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
            "# How much of the screen the game gets.\n\
             screen: 1280x720  # not fullscreen on this machine\n\
             \n\
             # Never show the ending.\n\
             skip_ending: false\n",
        );
        assert_eq!(
            config.screen,
            Screen::Window {
                width: 1280,
                height: 720
            }
        );
        assert!(!config.skip_ending);
        assert!(config.always_draw);
    }

    /// Anything this file does not know is an error rather than something passed over, since a
    /// setting that is quietly not read is a setting somebody thinks is on. What an option is
    /// named is not a key here, however much it reads like one, and neither is a key that used
    /// to be one — a file still carrying it is a file to edit rather than one to pass over.
    #[test]
    fn a_key_that_is_not_a_key_says_so() {
        for key in [
            "chapter_tuning",
            "own_frame_loop",
            "borderless",
            "block_replay_save",
            "own_score_file",
        ] {
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
            ("skip_ending: maybe\n", "expected a boolean"),
            ("skip_ending\n", "invalid type"),
            ("skip_ending: true\nskip_ending: false\n", "duplicate"),
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
