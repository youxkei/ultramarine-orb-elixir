//! The options for building the midstage table and for looking into a fault, which are
//! nobody's business but the person doing that.
//!
//! They are arguments rather than keys in `orb.yaml` because a file is for what somebody
//! sets once and plays with; these change from one launch to the next — a pass that collects
//! boundaries and a pass that judges them are the same session, ten seconds apart — and
//! editing a file to say which is a step that exists for no reason.
//!
//! The launcher takes them and hands them on to the game it starts, and the injected DLL
//! reads them back off its own command line, so the two sides always agree without either
//! writing anything down. 東方紅魔郷 never looks at its command line: `lpCmdLine` appears
//! once in the whole of `main.cpp`, as the parameter it ignores.

use std::fmt;

use crate::{Config, LogLevel};

/// What the launcher prints when it is asked, and when it is given something it cannot read.
pub const USAGE: &str = "\
usage: orb-launcher [option...]

Building the midstage chapter table, over a replay of a run:
  --collect            propose boundaries over the whole replay, at speed 64, with
                       nothing stopping and nobody at the keyboard
  --judge              step between them at speed 1 and decide about each: the pass
                       somebody watches
  --tune               propose and decide without either of those, for a run being played
  --replay             track chapters while a replay plays back
  --speed=N            updates per drawn frame while a replay plays back, or while a
                       run is being cleared

Getting to an ending, which only clearing the game reaches:
  --clear              nothing can hit the player and the run goes at 64 updates a
                       drawn frame, so half an hour of playing well is a minute of
                       holding the shot key. It leaves no record: no score file is
                       written, in the game's ranking or in orb's, and no replay
                       either. Add --no-chapters to spend none of it on snapshots of
                       a run nobody could have played

Looking into a fault:
  --log=LEVEL          quiet, normal or verbose
  --self-check         restore every snapshot as it is taken and report what differs
  --stress=N           restore the current chapter every N frames
  --no-chapters        leave orb loaded with none of its own work happening
  --no-memory          do not hook the heap, so a snapshot covers only .data
  --no-hooks           do not hook the game's frame at all

The keys pressed while a table is being built are fixed in the code, since whoever is
building one is the only person who presses them. Everything else — the window, the
ending, the joystick — is in orb.yaml, which is what somebody playing sets.\
";

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

fn bad(message: impl Into<String>) -> Error {
    Error { message: message.into() }
}

/// Whether `--help` was asked for, alongside the options that were read.
#[derive(Debug)]
pub struct Asked {
    pub help: bool,
}

impl Config {
    /// Applies the options to a config already read from the file. Both sides call this: the
    /// launcher with what it was given, the DLL with what it finds on the command line of the
    /// process it is inside.
    ///
    /// Anything not named here keeps the value the file's defaults gave it, which for every
    /// one of these is off, none or the key the table below names.
    pub fn take_arguments<'a>(
        &mut self,
        options: impl Iterator<Item = &'a str>,
    ) -> Result<Asked, Error> {
        let mut asked = Asked { help: false };
        // Set by whichever of the pass options came last, so that `--speed` after one of
        // them wins and before one of them does not.
        let mut pass_speed = None;
        let mut speed = None;
        for option in options {
            let (name, value) = match option.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (option, None),
            };
            match name {
                "--help" | "-h" => asked.help = true,
                // A pass over a replay is the whole of what it is: tuning, driven by the
                // replay, and one of the two kinds. Saying it in one word is the point.
                "--collect" => {
                    self.chapter_tuning = true;
                    self.during_replay = true;
                    self.chapter_stepping = false;
                    pass_speed = Some(COLLECT_SPEED);
                }
                "--judge" => {
                    self.chapter_tuning = true;
                    self.during_replay = true;
                    self.chapter_stepping = true;
                    pass_speed = Some(1);
                }
                "--tune" => self.chapter_tuning = true,
                "--replay" => self.during_replay = true,
                // The same shape as a pass: what it is for in one word, and a speed it
                // names rather than decides.
                "--clear" => {
                    self.fast_clear = true;
                    // Whatever the file says, since this is the other record such a run could
                    // leave behind and it would be a broken one: a replay holds the inputs and
                    // nothing about the player having been unhittable, so playing it back is a
                    // run that dies where this one did not.
                    self.block_replay_save = true;
                    pass_speed = Some(CLEAR_SPEED);
                }
                "--self-check" => self.self_check = true,
                "--no-chapters" => self.chapters = false,
                "--no-memory" => self.track_memory = false,
                "--no-hooks" => self.frame_hooks = false,
                "--speed" => speed = Some(number(name, value)?.max(1)),
                "--stress" => self.stress_restore_frames = number(name, value)?,
                "--log" => {
                    let level = value.ok_or_else(|| bad(format!("`{name}` needs =quiet, =normal or =verbose")))?;
                    self.log_level = LogLevel::parse(level).ok_or_else(|| {
                        bad(format!("`{name}={level}`: not one of quiet, normal or verbose"))
                    })?;
                }
                _ => return Err(bad(format!("unknown option `{option}`"))),
            }
        }
        if let Some(speed) = speed.or(pass_speed) {
            self.speed = speed;
        }
        Ok(asked)
    }

}

/// What `--collect` runs at: twenty minutes of a run in twenty seconds, and one frame in
/// sixty-four drawn, which is nothing to watch and everything to a pass nobody is watching.
const COLLECT_SPEED: u32 = 64;

/// What `--clear` runs at. The same number for a different reason: a run is being played, so
/// the frames still have to be watchable enough to aim at a boss, and one update a frame is
/// half an hour of them. Whoever is holding the shot key is not dodging anything — nothing can
/// hit them — so what is left to see is the run going by.
const CLEAR_SPEED: u32 = 64;

fn number(name: &str, value: Option<&str>) -> Result<u32, Error> {
    let value = value.ok_or_else(|| bad(format!("`{name}` needs a number, as {name}=64")))?;
    value.parse().map_err(|_| bad(format!("`{name}={value}`: not a number")))
}

/// The options out of a whole command line, which is what the DLL has: the words beginning
/// with two dashes and nothing else.
///
/// Everything before them is the path of whatever is being run, which on the game's own
/// command line is the game — a path can hold anything, including a word starting with a
/// dash, so what is taken is only what an option looks like.
pub fn options_in(command_line: &str) -> impl Iterator<Item = &str> {
    command_line.split_whitespace().filter(|word| word.starts_with("--"))
}

#[cfg(test)]
mod tests {
    use crate::{Config, LogLevel};

    fn with(options: &str) -> Config {
        let mut config = Config::parse(std::path::Path::new("/opt/orb/orb.yaml"), "").unwrap();
        config.take_arguments(super::options_in(options)).unwrap();
        config
    }

    /// The two passes, each said in one word: what they are for is the difference between a
    /// pass nobody watches and a pass somebody does.
    #[test]
    fn a_pass_is_one_word() {
        let collect = with(r"C:\game\th06.exe --collect");
        assert!(collect.chapter_tuning && collect.during_replay);
        assert!(!collect.chapter_stepping);
        assert_eq!(collect.speed, 64);

        let judge = with("--judge");
        assert!(judge.chapter_tuning && judge.during_replay && judge.chapter_stepping);
        assert_eq!(judge.speed, 1);
    }

    /// A speed given by hand wins over the one the pass chose, whichever order they come in:
    /// the pass names a default, not a decision.
    #[test]
    fn a_speed_given_by_hand_wins() {
        assert_eq!(with("--collect --speed=8").speed, 8);
        assert_eq!(with("--speed=8 --collect").speed, 8);
    }

    /// A clear is one word for the same reason a pass is, and the speed that comes with it is a
    /// default in the same way: named, not decided.
    #[test]
    fn a_clear_is_one_word() {
        let clear = with("--clear");
        assert!(clear.fast_clear);
        assert_eq!(clear.speed, 64);
        assert_eq!(with("--clear --speed=1").speed, 1);
    }

    /// It leaves no record of a run nobody could have played, whichever way the file has that
    /// set: a replay of one plays back as a run that dies where this one did not.
    #[test]
    fn a_clear_writes_no_replay() {
        let mut config = Config::parse(std::path::Path::new("/opt/orb/orb.yaml"), "").unwrap();
        config.block_replay_save = false;
        config.take_arguments(super::options_in("--clear")).unwrap();
        assert!(config.block_replay_save);
    }

    /// Nothing else about the run changes. What a clear is for is reaching an ending, and
    /// whether the chapters of a run nobody could have played are worth snapshotting is a
    /// separate question with `--no-chapters` for an answer.
    #[test]
    fn a_clear_changes_nothing_but_the_player_the_speed_and_what_is_written() {
        let clear = with("--clear");
        assert!(clear.chapters && clear.skip_ending);
        assert!(!clear.chapter_tuning && !clear.during_replay && !clear.self_check);
    }

    #[test]
    fn nothing_given_leaves_the_file_alone() {
        let plain = with(r"C:\game\th06.exe");
        assert!(!plain.chapter_tuning && !plain.during_replay && !plain.chapter_stepping);
        assert!(!plain.fast_clear);
        assert_eq!(plain.log_level, LogLevel::Normal);
        assert_eq!(plain.speed, 1);
    }

    #[test]
    fn what_cannot_be_read_says_so() {
        let mut config = Config::parse(std::path::Path::new("/opt/orb/orb.yaml"), "").unwrap();
        for option in ["--nonsense", "--speed", "--speed=fast", "--log=loud"] {
            let error = config.take_arguments(super::options_in(option)).unwrap_err();
            assert!(error.message.contains(option.split('=').next().unwrap()), "{error}");
        }
    }
}
