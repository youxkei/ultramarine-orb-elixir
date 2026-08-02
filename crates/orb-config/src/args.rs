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
//!
//! Both sides read them with the same [`Options`]: the launcher as one part of its own
//! arguments, alongside the `--config` that is its business alone, and the DLL out of the
//! command line of the process it is inside.

use clap::{Args, Parser};

use crate::{Config, LogLevel};

/// The three reasons to give an option at all, which are what `--help` lists them under.
const TABLE: &str = "Building the midstage chapter table, over a replay of a run";
const ENDING: &str = "Getting to an ending, which only clearing the game reaches";
const FAULT: &str = "Looking into a fault";

/// The last thing `--help` says, which is where the other half of the settings are. It lives
/// beside the options it is about rather than in the launcher that prints it.
pub const AFTER_HELP: &str = "\
The keys pressed while a table is being built are fixed in the code, since whoever is building \
one is the only person who presses them. Everything else — the window, the ending, the score \
file — is in orb.yaml, which is what somebody playing sets.";

/// What is said at a launch rather than left in `orb.yaml`.
///
/// A value is written `--speed=64` and not `--speed 64`. The DLL takes its options out of the
/// game's whole command line by picking the words that begin with two dashes, so a value in a
/// word of its own would be dropped on that side and the two would read the same line
/// differently; requiring the `=` makes the line that cannot be read that way unwritable.
#[derive(Args, Debug)]
pub struct Options {
    /// propose boundaries over the whole replay, at speed 64, with nothing stopping and
    /// nobody at the keyboard
    // Against --judge rather than the later of the two winning: they are two passes over the
    // same replay, and asking for both is a mistake worth being told about, not a preference.
    #[arg(long, conflicts_with = "judge", help_heading = TABLE)]
    collect: bool,

    /// step between them at speed 1 and decide about each: the pass somebody watches
    #[arg(long, help_heading = TABLE)]
    judge: bool,

    /// propose and decide without either of those, for a run being played
    #[arg(long, help_heading = TABLE)]
    tune: bool,

    /// track chapters while a replay plays back
    #[arg(long, help_heading = TABLE)]
    replay: bool,

    /// updates per drawn frame while a replay plays back, or while a run is being cleared
    #[arg(
        long,
        require_equals = true,
        value_name = "N",
        value_parser = clap::value_parser!(u32).range(1..),
        help_heading = TABLE,
    )]
    speed: Option<u32>,

    /// nothing can hit the player and the run goes at 64 updates a drawn frame, so half an
    /// hour of playing well is a minute of holding the shot key. It leaves no record: no score
    /// file is written, in the game's ranking or in orb's, and no replay either. Add
    /// --no-chapters to spend none of it on snapshots of a run nobody could have played
    #[arg(long, help_heading = ENDING)]
    clear: bool,

    /// how much goes into orb.log; normal unless said
    #[arg(long, require_equals = true, value_name = "LEVEL", help_heading = FAULT)]
    log: Option<LogLevel>,

    /// write what every frame that missed the cadence spent its turn on, whatever --log says.
    /// Its own switch because what the log writes is one of the things that makes a frame
    /// late, so this goes with --log=quiet: nothing in the file but the pacing
    #[arg(long, help_heading = FAULT)]
    pacing: bool,

    /// pin at N microseconds the time left for the compositor to draw in, between the frame
    /// being handed over and the blank it is to be shown at, instead of finding it while
    /// running. It will not say what it needs, so this is swept: small enough that frames are
    /// known to miss their blank, then up until they stop
    #[arg(long, require_equals = true, value_name = "N", help_heading = FAULT)]
    compose: Option<u32>,

    /// restore every snapshot as it is taken and report what differs
    #[arg(long, help_heading = FAULT)]
    self_check: bool,

    /// restore the current chapter every N frames
    #[arg(long, require_equals = true, value_name = "N", help_heading = FAULT)]
    stress: Option<u32>,

    /// leave orb loaded with none of its own work happening
    #[arg(long, help_heading = FAULT)]
    no_chapters: bool,

    /// do not hook the heap, so a snapshot covers only .data
    #[arg(long, help_heading = FAULT)]
    no_memory: bool,

    /// do not hook the game's frame at all
    #[arg(long, help_heading = FAULT)]
    no_hooks: bool,
}

/// A whole command line with the options somewhere in it, which is the shape the DLL is given
/// them in: it has the line the launcher started the game with and nothing else.
#[derive(Parser, Debug)]
#[command(name = "orb")]
struct CommandLine {
    #[command(flatten)]
    options: Options,
}

impl Options {
    /// The options out of a whole command line, which is what the DLL has.
    ///
    /// Everything before them is the path of whatever is being run, which on the game's own
    /// command line is the game — a path can hold anything, including a word starting with a
    /// dash, so what is taken is only what an option looks like.
    pub fn from_command_line(command_line: &str) -> Result<Self, clap::Error> {
        let words = command_line
            .split_whitespace()
            .filter(|word| word.starts_with("--"));
        CommandLine::try_parse_from(std::iter::once("orb").chain(words)).map(|line| line.options)
    }
}

impl Config {
    /// Applies the options to a config already read from the file. Both sides call this: the
    /// launcher with what it was given, the DLL with what it finds on the command line of the
    /// process it is inside.
    ///
    /// Anything not given keeps the value the file's defaults gave it, which for every one of
    /// these is off, none or one.
    pub fn apply(&mut self, options: &Options) {
        // Named by whichever pass was asked for, so that a `--speed` given by hand wins
        // whichever order the two come in: the pass names a default, not a decision.
        let mut pass_speed = None;
        // A pass over a replay is the whole of what it is: tuning, driven by the replay, and
        // one of the two kinds. Saying it in one word is the point.
        if options.collect {
            self.chapter_tuning = true;
            self.during_replay = true;
            pass_speed = Some(COLLECT_SPEED);
        }
        if options.judge {
            self.chapter_tuning = true;
            self.during_replay = true;
            self.chapter_stepping = true;
            pass_speed = Some(1);
        }
        if options.tune {
            self.chapter_tuning = true;
        }
        if options.replay {
            self.during_replay = true;
        }
        // The same shape as a pass: what it is for in one word, and a speed it names rather
        // than decides.
        if options.clear {
            self.fast_clear = true;
            // Whatever the file says, since this is the other record such a run could leave
            // behind and it would be a broken one: a replay holds the inputs and nothing about
            // the player having been unhittable, so playing it back is a run that dies where
            // this one did not.
            self.block_replay_save = true;
            pass_speed = Some(CLEAR_SPEED);
        }
        if let Some(speed) = options.speed.or(pass_speed) {
            self.speed = speed;
        }
        if let Some(level) = options.log {
            self.log_level = level;
        }
        if options.pacing {
            self.pacing_log = true;
        }
        if let Some(compose_us) = options.compose {
            self.compose_us = compose_us;
        }
        if options.self_check {
            self.self_check = true;
        }
        if let Some(frames) = options.stress {
            self.stress_restore_frames = frames;
        }
        if options.no_chapters {
            self.chapters = false;
        }
        if options.no_memory {
            self.track_memory = false;
        }
        if options.no_hooks {
            self.frame_hooks = false;
        }
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

#[cfg(test)]
mod tests {
    use super::Options;
    use crate::{Config, LogLevel};

    fn config() -> Config {
        Config::parse(std::path::Path::new("/opt/orb/orb.yaml"), "").unwrap()
    }

    fn with(command_line: &str) -> Config {
        let mut config = config();
        config.apply(&Options::from_command_line(command_line).unwrap());
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

    /// Asking for both passes at once is a mistake rather than the later one winning: they are
    /// two passes over the same replay, made of opposite answers to whether it stops.
    #[test]
    fn the_two_passes_are_not_asked_for_together() {
        assert!(Options::from_command_line("--collect --judge").is_err());
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
        let mut config = config();
        config.block_replay_save = false;
        config.apply(&Options::from_command_line("--clear").unwrap());
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

    /// The pacing is asked for on its own, and `quiet` is the level to ask for it at: what
    /// the log writes is one of the reasons a frame misses its blank, so a run watching the
    /// pacing wants nothing else writing.
    #[test]
    fn the_pacing_is_asked_for_apart_from_the_level() {
        let quiet = with("--pacing --log=quiet");
        assert!(quiet.pacing_log);
        assert_eq!(quiet.log_level, LogLevel::Quiet);
        assert!(!with("--log=verbose").pacing_log);
    }

    /// The compositor's drawing time is swept, so it is given in the microseconds a sweep
    /// steps in, and not given at all when it is to be found while running.
    #[test]
    fn the_compositors_drawing_time_is_pinned_in_microseconds() {
        assert_eq!(with("--compose=200").compose_us, 200);
        assert_eq!(with("--pacing").compose_us, 0);
    }

    #[test]
    fn nothing_given_leaves_the_file_alone() {
        let plain = with(r"C:\game\th06.exe");
        assert!(!plain.chapter_tuning && !plain.during_replay && !plain.chapter_stepping);
        assert!(!plain.fast_clear && !plain.pacing_log);
        assert_eq!(plain.log_level, LogLevel::Normal);
        assert_eq!(plain.speed, 1);
    }

    /// A value is written onto the option, so that a line the launcher accepts is a line the
    /// DLL reads the same way: the DLL takes the options out of the game's command line by the
    /// two dashes they start with, and would never see a value standing in a word of its own.
    #[test]
    fn a_value_is_written_onto_the_option() {
        use clap::Parser;

        assert!(super::CommandLine::try_parse_from(["orb", "--speed=8"]).is_ok());
        assert!(super::CommandLine::try_parse_from(["orb", "--speed", "8"]).is_err());
        assert!(super::CommandLine::try_parse_from(["orb", "--log", "verbose"]).is_err());
    }

    #[test]
    fn what_cannot_be_read_says_so() {
        for option in [
            "--nonsense",
            "--speed",
            "--speed=fast",
            "--speed=0",
            "--log=loud",
        ] {
            let error = Options::from_command_line(option).unwrap_err();
            let name = option.split('=').next().unwrap();
            assert!(error.to_string().contains(name), "{option}: {error}");
        }
    }
}
