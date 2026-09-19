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
//! writing anything down. th06 never looks at its command line: `lpCmdLine` appears
//! once in the whole of `main.cpp`, as the parameter it ignores.
//!
//! Both sides read them with the same [`Options`]: the launcher as one part of its own
//! arguments, alongside the `--config` that is its business alone, and the DLL out of the
//! command line of the process it is inside.
//!
//! Everything to do with building the midstage table, reaching an ending, or looking into a fault is an
//! argument to `orb.exe` — `--help` lists them — because a file is the wrong place for something that is
//! different every time it is run. The two passes over a replay are one word each:
//!
//! | | |
//! | --- | --- |
//! | `--collect` | propose boundaries over the whole replay, at 64 updates a frame, with nothing stopping and nobody at the keyboard |
//! | `--judge` | step between them at one update a frame and decide about each: the pass somebody watches |
//!
//! and beside them `--tune`, `--replay`, `--speed=N`, `--log=quiet|normal|verbose`, `--compose=N`,
//! `--self-check`, `--stress=N`, `--sent-keys`, and `--no-pacing`, `--no-chapters`, `--no-memory`,
//! `--no-resume`, `--no-frame-loop` and `--no-hooks` for taking orb apart until a fault stops happening.
//! What each does is the doc of the module that does it: the passes and the keys `orb_core::tuning`, the
//! pacing's switches `orb_core::frame`, `--sent-keys` `orb_core::input`, `--self-check` and `--stress`
//! `orb_core::snapshot`, `--clear` and the ending `orb_core::runtime`, `--no-resume` `orb_core::resume`.
//!
//! One clap definition for all of them, read by both halves: the launcher out of its own arguments, the DLL
//! out of the command line the launcher wrote. A value goes onto its option, `--speed=64` rather than
//! `--speed 64`, and clap is told to require the `=` — the DLL picks the options out of that whole command
//! line by the two dashes they begin with, since everything before them is a path that may hold anything,
//! so a value standing in a word of its own is a line the two halves would read differently.

use std::path::PathBuf;

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
one is the only person who presses them. Everything else — the window, the ending, the wash a \
chapter gets — is asked for before the game starts and kept in orb.yaml; --settings asks again. \
Whether a run keeps chapters at all is asked inside the game, where the run is started.";

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
    // **And not one a recipe drives.** A bot's word is folded into the pad orb reads its own keys off — see
    // `runtime::pad` — and the key that puts a boundary down is that pad's shot, which a bot holds and lets go
    // of for one frame in sixty. Every release is then an edge: a judging pass with a recipe wrote a boundary a
    // second, twelve into the Extra's first nine seconds before anybody had touched a thing. The window is the
    // other half — a launch that plays itself is put in a corner at a size of its own, and a judging pass is
    // watched — and neither half is worth an option, the front end being four presses by hand.
    #[arg(long, conflicts_with = "autoplay", help_heading = TABLE)]
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

    /// game frames a sixtieth of a second is paced to hold, so that walking a run to some stage
    /// takes minutes rather than half an hour. **Everything orb does per frame is what it does in
    /// an ordinary launch** — this shortens the wait between frames and nothing else, which is what
    /// makes it different from --speed. The clock paces it, no display having a blank per frame to
    /// give at four times sixty, and the music plays at its own speed while the stage goes by
    /// quicker: a check of the sound is a launch of this at 1. **And for a game orb has no frame of
    /// its own in, what runs faster is the clock the game waits on** — orb answers its ask for
    /// timeGetTime, GetTickCount and QueryPerformanceCounter with one running this many times over
    #[arg(
        long,
        require_equals = true,
        value_name = "N",
        value_parser = clap::value_parser!(u32).range(1..),
        help_heading = TABLE,
    )]
    pace: Option<u32>,

    /// say which addresses of the game's own move with the direction being pressed, for a game
    /// nobody has read a field of. Hold one direction for a while and then the other — a count of
    /// frames rises on every frame of a pass that wants rising, and only what is being steered
    /// rises and then falls. Every surviving address is said with its value, and the pass is nine
    /// megabytes a frame, so this is for a launch that is looking and no other
    #[arg(long, help_heading = TABLE)]
    find: bool,

    /// nothing can hit the player and the run goes at 64 updates a drawn frame, so half an
    /// hour of playing well is a minute of holding the shot key. It leaves no record: no score
    /// file is written, in the game's ranking or in orb's, no replay, and no chapter for a
    /// later launch to offer. Add --no-chapters to spend none of it on snapshots of a run
    /// nobody could have played
    #[arg(long, help_heading = ENDING)]
    clear: bool,

    /// how much goes into orb.log; normal unless said
    #[arg(long, require_equals = true, value_name = "LEVEL", help_heading = FAULT)]
    log: Option<LogLevel>,

    /// stop writing what every frame that missed the cadence spent its turn on, which is
    /// written at whatever --log says. What it takes back is one question of the compositor a
    /// frame and a line per frame that missed, neither of them out of the frame's own time
    #[arg(long, help_heading = FAULT)]
    no_pacing: bool,

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

    /// put back only the first N regions of every snapshot, which is `--no-memory`'s bisect made finer
    ///
    /// **For a fault the *contents* of one region are suspected of** rather than the writing of them.
    /// `--no-memory` narrows a snapshot to `.data` before it is taken; this leaves the snapshot whole and
    /// holds the restore to the first N of what it saved, so the regions are bisected one launch at a
    /// time with everything else about the run unchanged. th06nc is what it was added for: it dies on the
    /// first update *after* a restore whose every region was written — the log reaches
    /// `restore: putting back 8 of 8` and the lines past it — so what is left to find is which region's
    /// bytes the update could not survive.
    ///
    /// **A launch under it is a game deliberately half put back**, and nothing a run anybody plays wants.
    #[arg(long, require_equals = true, value_name = "N", help_heading = FAULT)]
    put_back_only: Option<u32>,

    /// leave the first N regions of every snapshot alone and put back the rest, which is the other half of
    /// `--put-back-only`
    ///
    /// **Both halves exist because the regions are not independent**: `.data` is the first of them and
    /// holds pointers into the heap blocks after it, so `--put-back-only=1` is a game whose chain names
    /// blocks that have moved and dies of the flag rather than of the fault. `--put-back-skipping=1` is
    /// the informative turn — the heap put back and `.data` left as the run had it — and which of the two
    /// a fault is in is what the pair answers between them.
    #[arg(long, require_equals = true, value_name = "N", help_heading = FAULT)]
    put_back_skipping: Option<u32>,

    /// have a restore write back even the ranges it would hold back, which is the third of the bisects
    ///
    /// **What it asks is what the mixture costs.** A restore leaves most of a region as the snapshot had
    /// it and the ranges of `Game::live_handles` and the audio's as the run has them — eleven of them and
    /// 195,276 bytes over th06nc — and that game dies of `.data` coming back while living when `.data` is
    /// left alone entirely, so whether the *mixture* is what its next update cannot survive is the one
    /// question between those two answers. **A restore under it puts back a handle the game has since
    /// replaced**, which is what the holes exist to stop.
    #[arg(long, help_heading = FAULT)]
    hold_back_nothing: bool,

    /// hold back OFFSET+LEN of the game's memory as well, in hex, given once per span; SOURCE: in front
    /// names which of the things the game took — data, heap1.3, reserve2 — and data is the default
    ///
    /// **Which is how a bisection inside the game's memory is written.** The exe is built with ASLR — a
    /// static written down as `0x140aa1e90` lives wherever the loader put the image — so what names a span
    /// across launches is an offset and not an address. th06nc's `.data` begins at `0x140323000`, so its
    /// enemy manager is `--hold-back-at=77ee90+10c100`.
    ///
    /// **And the source goes in front where the span is not `.data`'s**: `--hold-back-at=heap1.3:0+7800`
    /// is the first `0x7800` bytes of the third region of the first heap the game allocated from, which is
    /// at a different address every launch where the region is the same segment. The names are
    /// `orb_api::mem::Source`'s, and the log line `memory: saving` puts one in front of every region a
    /// snapshot holds.
    ///
    /// **A source and not a place in the region list**, which this named first: the committed runs of one
    /// heap region are as many entries in that list as it has holes, and one launch of th06nc listed the
    /// game's heap as 8 entries in one chapter and 9 in another — so `2:0+7800` named different pages
    /// within the one run it was meant to bisect.
    ///
    /// **And a heap region and not a heap**, since an offset into a heap names nothing twice either: the
    /// distances between a heap's segments are the OS's, and one launch of th06nc put the segment another
    /// had `0x1a670000` above the first `0x180000` below it.
    ///
    /// **What it is for is what `--put-back-only` and `--put-back-skipping` leave**: those name a count of
    /// regions and this names a span inside one source. th06nc's chapter retries died of one byte of
    /// `.data`, found by halving with this flag; what they die of in stage 2 is somewhere in the game's
    /// heap, which is where the source earns its place.
    #[arg(
        long,
        require_equals = true,
        value_name = "[SOURCE:]OFFSET+LEN",
        help_heading = FAULT
    )]
    hold_back_at: Vec<String>,

    /// leave orb loaded with none of its own work happening
    #[arg(long, help_heading = FAULT)]
    no_chapters: bool,

    /// do not hook the heap, so a snapshot covers only .data
    ///
    /// **Which makes it the bisect for a fault a restore is suspected of**, and the three flags are run as
    /// one: `--self-check --stress=600` restores every snapshot as it is taken and names every region that
    /// does not come back byte for byte; `--no-memory` beside it narrows the snapshot to `.data`, so a
    /// fault that goes away with it is a heap block the restore writes back; and `--no-chapters` takes the
    /// restore out of the launch altogether, so a fault that survives *that* is not orb's restore at all.
    /// One fault of th07's was ruled out that way — it came back with `--no-chapters`, on a different heap
    /// block every run.
    #[arg(long, help_heading = FAULT)]
    no_memory: bool,

    /// have the game read its keyboard the way it does when DirectInput has no device, so that keys
    /// another program sends with SendInput are seen. For driving a session from a script, which is
    /// otherwise impossible: the device the game takes is exclusive and foreground, and does not see
    /// them
    #[arg(long, help_heading = FAULT)]
    sent_keys: bool,

    /// play the run itself, to the recipe in this file: a list of steps, each a word to hold and how
    /// many frames to hold it for, and then the run played out of what the game holds — the shot
    /// held, and the bomb the frame the player is hit. For having a run to pick up out of a file
    /// without playing one; nothing is made invulnerable and every record a played run leaves is
    /// left, so what it leaves behind is a run
    #[arg(long, require_equals = true, value_name = "RECIPE", help_heading = FAULT)]
    autoplay: Option<PathBuf>,

    /// write down what every frame held — the run's state as the game's memory has it, the word it
    /// was played with, which question of orb's own was up, and the log's lines — as MessagePack
    /// records to this file, beside the game where the path names no directory, for a check to read
    /// after the run. The file is started over each launch
    #[arg(long, require_equals = true, value_name = "FILE", help_heading = FAULT)]
    trace: Option<PathBuf>,

    /// keep the files orb writes of its own — the run left unfinished, its replay, the name it
    /// remembers, and a trace naming no directory — under this directory rather than beside the
    /// game. For a launch that is a test: what an earlier session left beside the game then
    /// answers none of this launch's questions, and nothing there has to be moved to make room
    #[arg(long, require_equals = true, value_name = "DIR", help_heading = FAULT)]
    keep_in: Option<PathBuf>,

    /// answer the imports a game sold through Steam reaches it through, so that the launch needs no
    /// client running and puts nothing on anybody's account. What a launch playing itself does anyway,
    /// so this is for one somebody is playing; `without_steam` in `orb.yaml` is the same answer kept
    #[arg(long, help_heading = FAULT)]
    without_steam: bool,

    /// do not write down what a run pressed, so no chapter can be picked up in a later launch.
    /// Takes the game's input read and the moment a stage's numbers are put in place out of
    /// orb's hands, and leaves pointdevice_resume/ alone
    #[arg(long, help_heading = FAULT)]
    no_resume: bool,

    /// leave the frame to the game: its own order, draw before update, and its own pacing,
    /// with the update and the draw still hooked so chapters carry on. A frame of input lag
    /// comes back with it, and frames are doubled and dropped as they were
    #[arg(long, help_heading = FAULT)]
    no_frame_loop: bool,

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

    /// **Whether this launch plays the run itself**, which is `--autoplay` naming a recipe.
    ///
    /// Asked before the recipe is read, and so of the option rather than of [`Config::autoplay`]: a file
    /// that would not open leaves that as nothing, and a launch nobody is sitting at is still a launch
    /// nobody is sitting at — see the settings dialog in the launcher.
    pub fn plays_itself(&self) -> bool {
        self.autoplay.is_some()
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
            // The other record such a run could leave behind, and it would be a broken one: a
            // replay holds the inputs and nothing about the player having been unhittable, so
            // playing it back is a run that dies where this one did not. A cleared run reaches
            // the screen that offers to save one — a pointdevice run does not — so the write
            // itself is what has to be refused.
            self.block_replay_save = true;
            // And the third record such a run could leave, for the same reason as the replay:
            // what is written down is the buttons and nothing about the player having been
            // unhittable, so a later launch offered that chapter plays them into a player who
            // dies — and lands somewhere else, which is what `resume: the player was hit at
            // frame N` then reports about a file this run had no business writing.
            self.resume = false;
            pass_speed = Some(CLEAR_SPEED);
        }
        if let Some(pace) = options.pace {
            self.pace = pace;
        }
        if options.find {
            self.find = true;
        }
        if let Some(speed) = options.speed.or(pass_speed) {
            self.speed = speed;
        }
        if let Some(level) = options.log {
            self.log_level = level;
        }
        if options.no_pacing {
            self.pacing_log = false;
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
        if let Some(regions) = options.put_back_only {
            self.put_back_only = regions;
        }
        if let Some(regions) = options.put_back_skipping {
            self.put_back_skipping = regions;
        }
        if options.hold_back_nothing {
            self.hold_back_nothing = true;
        }
        // **A span that will not parse is said and dropped rather than refused**: what asks for these is a
        // launch bisecting a fault, and a launch that came up with one span of three is a launch whose log
        // says which.
        for span in &options.hold_back_at {
            // **The source is optional and `data` where it is left out**, so that every span written
            // before it existed still means what it meant: those named `.data`, which was the first
            // region then and is the first source now.
            //
            // **And taken as written rather than held against a list of names**, which is where the
            // answer is: whether a snapshot saved anything of `heap3` is the snapshot's to say, and it
            // says so in the log — `restore: this snapshot saved nothing of heap3`. A list here would be
            // the same names written twice, and the copy that is not `orb_api::mem::Source` is the one
            // that goes stale.
            let (source, rest) = match span.split_once(':') {
                Some(("", _)) => {
                    eprintln!("orb: {span} names no source before its colon; it is dropped");
                    continue;
                }
                Some((which, rest)) => (which, rest),
                None => ("data", span.as_str()),
            };
            match rest.split_once('+') {
                Some((at, len)) => {
                    match (
                        usize::from_str_radix(at.trim_start_matches("0x"), 16),
                        usize::from_str_radix(len.trim_start_matches("0x"), 16),
                    ) {
                        (Ok(at), Ok(len)) => {
                            self.hold_back_at.push((source.to_owned(), at, len));
                        }
                        _ => eprintln!("orb: {span} is not OFFSET+LEN in hex; it is dropped"),
                    }
                }
                None => eprintln!("orb: {span} has no + in it; it is dropped"),
            }
        }
        if options.no_chapters {
            self.chapters = false;
        }
        if options.no_memory {
            self.track_memory = false;
        }
        if options.no_resume {
            self.resume = false;
        }
        if options.sent_keys {
            self.sent_keys = true;
        }
        // Read here rather than carried as a path, so that what reaches a game is the recipe itself: a path
        // handed on would be a second thing to have gone wrong by the time anything could say so, and the
        // launcher is where a file that will not open is still worth a word about. A recipe that cannot be
        // read at all leaves this as nothing, which is a launch that plays nothing and says why.
        if let Some(recipe) = &options.autoplay {
            match std::fs::read_to_string(recipe) {
                Ok(text) => self.autoplay = Some(text),
                Err(why) => eprintln!("orb: {} cannot be read: {why}", recipe.display()),
            }
            // **A launch that plays itself is a launch nobody is watching**, so the log is the only place
            // anything about it can be seen — and everything a bot says about how it played is detail by nature:
            // where it was hit, what its model was made of, which item of a menu the cursor is on. Asking for
            // the harness is asking for what the harness sees, so the level comes with it rather than having to
            // be remembered beside it.
            self.log_level = LogLevel::Verbose;
        }
        if let Some(path) = &options.trace {
            self.trace = Some(path.clone());
        }
        // The settings themselves were read from beside the game already, and stay the game's: what moves
        // is where this launch *writes*, which is every path derived from the base directory.
        if let Some(dir) = &options.keep_in {
            self.base_dir = dir.clone();
        }
        // **Only ever said and never unsaid**, which is the shape every switch here has: what says the
        // other answer is the file, and what says neither is a launch playing itself — see
        // `Config::without_steam_wanted`.
        if options.without_steam {
            self.without_steam = Some(true);
        }
        if options.no_frame_loop {
            self.own_frame_loop = false;
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

    /// **A launch being checked says where its trace goes and where its files are kept**, and neither is
    /// anything a launch nobody asked has: a settings file cannot name them.
    #[test]
    fn a_trace_and_a_directory_to_keep_files_in_are_only_ever_asked_for() {
        let plain = with("");
        assert_eq!(plain.trace, None);
        assert_eq!(plain.base_dir, std::path::Path::new("/opt/orb"));

        let checked = with(r"--trace=cases\one\trace.msgpack --keep-in=C:\copy\orb-real\1\base");
        assert_eq!(
            checked.trace.as_deref(),
            Some(std::path::Path::new(r"cases\one\trace.msgpack"))
        );
        assert_eq!(
            checked.base_dir,
            std::path::Path::new(r"C:\copy\orb-real\1\base")
        );
        // What was read stays read from beside the game: only where this launch writes has moved.
        assert_eq!(checked.game_dir, std::path::Path::new("/opt/orb"));
    }

    /// Asking for both passes at once is a mistake rather than the later one winning: they are
    /// two passes over the same replay, made of opposite answers to whether it stops.
    #[test]
    fn the_two_passes_are_not_asked_for_together() {
        assert!(Options::from_command_line("--collect --judge").is_err());
    }

    /// **And a judging pass is not driven by a recipe.** A bot's word is folded into the pad orb reads its own
    /// keys off, and the key that puts a boundary down is that pad's shot: a bot holds the shot and lets it go
    /// for one frame in sixty, so every release is an edge and a judging pass with a recipe wrote a boundary a
    /// second — twelve of them into the first nine seconds of the Extra before anybody touched anything. The
    /// window is the other half of it, a launch that plays itself being put in a corner at a size of its own,
    /// where what a judging pass is for is being watched.
    #[test]
    fn a_judging_pass_is_not_driven_by_a_recipe() {
        assert!(Options::from_command_line("--judge --autoplay=recipe.yaml").is_err());
        assert!(Options::from_command_line("--autoplay=recipe.yaml --judge").is_err());
        // The collecting pass is nobody's to watch and nothing of it reads a pad, so a recipe is what starts
        // the replay there.
        assert!(Options::from_command_line("--collect --autoplay=recipe.yaml").is_ok());
    }

    /// A speed given by hand wins over the one the pass chose, whichever order they come in:
    /// the pass names a default, not a decision.
    #[test]
    fn a_speed_given_by_hand_wins() {
        assert_eq!(with("--collect --speed=8").speed, 8);
        assert_eq!(with("--speed=8 --collect").speed, 8);
    }

    /// **The finer of the two bisects a restore has**, and nought where nobody asked for it: a launch
    /// whose restore puts back part of what it saved is a game deliberately left half restored, so it is
    /// the flag that says so and never a default.
    #[test]
    fn a_restore_puts_back_every_region_unless_a_launch_says_otherwise() {
        assert_eq!(with("").put_back_only, 0);
        assert_eq!(with("--put-back-only=1").put_back_only, 1);
        assert_eq!(with("--put-back-only=4").put_back_only, 4);
        // And it is the snapshot's own regions it counts, not the heap's being hooked at all — which is
        // `--no-memory`, the coarse half, and the two are independent.
        let both = with("--put-back-only=1 --no-memory");
        assert!(both.put_back_only == 1 && !both.track_memory);
        // **And the same bisect from the other end**, which is the half that says anything where `.data`
        // is the region being left out: a prefix holding it without the heap it names is inconsistent by
        // construction.
        assert_eq!(with("").put_back_skipping, 0);
        assert_eq!(with("--put-back-skipping=1").put_back_skipping, 1);
        // **And the third of them**, which is about the ranges inside a region rather than the regions:
        // a restore holds some back and writing them too is what says whether the mixture is the fault.
        assert!(!with("").hold_back_nothing);
        assert!(with("--hold-back-nothing").hold_back_nothing);
        // **And the spans inside `.data`**, in hex and as offsets, which is what a bisection of nine
        // megabytes of statics is written in: `0x140aa1e90 - 0x140323000` is th06nc's enemy manager. A
        // span with no source named is that one's, `.data` being what every span written before the
        // source existed meant.
        assert!(with("").hold_back_at.is_empty());
        assert_eq!(
            with("--hold-back-at=77ee90+10c100").hold_back_at,
            [("data".to_owned(), 0x77_ee90, 0x10_c100)],
        );
        assert_eq!(
            with("--hold-back-at=0x10+0x20 --hold-back-at=30+40").hold_back_at,
            [
                ("data".to_owned(), 0x10, 0x20),
                ("data".to_owned(), 0x30, 0x40)
            ],
        );
        // **And a source in front of the offset**, which is what a span outside `.data` has to be named
        // by: a region of the game's heap is at a different address every launch, where the segment the
        // walk hands over third is the same segment. The dot is nothing to this parse, which takes the
        // name as written — `orb_api::mem::Source` is where it is spelled.
        assert_eq!(
            with("--hold-back-at=heap1.3:0+7800").hold_back_at,
            [("heap1.3".to_owned(), 0x0, 0x7800)],
        );
        // **A name no source has is carried as written**, and which sources a snapshot saved is answered
        // where that is known, by the restore's own log line. Here it would be a second list of the same
        // names, and the copy that is not `orb_api::mem::Source` is the one that goes stale.
        assert_eq!(
            with("--hold-back-at=heap9.1:0+10").hold_back_at,
            [("heap9.1".to_owned(), 0x0, 0x10)],
        );
        // A span that will not parse is dropped and said, a launch bisecting a fault being better served
        // by the two spans it did get than by no launch at all. A colon with nothing in front of it is one
        // of those, naming no source at all.
        assert!(with("--hold-back-at=nonsense").hold_back_at.is_empty());
        assert!(with("--hold-back-at=zz+1").hold_back_at.is_empty());
        assert!(with("--hold-back-at=:10+20").hold_back_at.is_empty());
        assert!(with("--hold-back-at=data:zz+1").hold_back_at.is_empty());
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

    /// It leaves no record of a run nobody could have played: whether the record is a replay or
    /// the chapter written down for a later launch, what it holds is the buttons and nothing
    /// about the player having been unhittable, so playing either back is a run that dies where
    /// this one did not.
    #[test]
    fn a_clear_writes_no_replay_and_no_chapter_to_pick_up() {
        assert!(!config().block_replay_save);
        assert!(with("--clear").block_replay_save);
        assert!(config().resume);
        assert!(!with("--clear").resume);
    }

    /// Nothing else about the run changes. What a clear is for is reaching an ending, and
    /// whether the chapters of a run nobody could have played are worth snapshotting is a
    /// separate question with `--no-chapters` for an answer — the snapshots a retry uses are
    /// not the file a later launch would be offered.
    #[test]
    fn a_clear_changes_nothing_but_the_player_the_speed_and_what_is_written() {
        let clear = with("--clear");
        assert!(clear.chapters && clear.skip_ending);
        assert!(!clear.chapter_tuning && !clear.during_replay && !clear.self_check);
    }

    /// The pacing is written unless the launch says not to, and it is not a tier of the level:
    /// what a run spends its frames on is a different question from how the run is going, so
    /// neither level answers it and every level keeps it.
    #[test]
    fn the_pacing_is_written_unless_the_launch_says_not_to() {
        assert!(with("").pacing_log);
        let quiet = with("--log=quiet");
        assert!(quiet.pacing_log);
        assert_eq!(quiet.log_level, LogLevel::Quiet);
        assert!(!with("--no-pacing").pacing_log);
        assert!(!with("--no-pacing --log=verbose").pacing_log);
    }

    /// What a run pressed is written down unless the launch says not to, and the file cannot say
    /// it: a run that cannot be picked up again loses the one thing orb exists for the moment the
    /// game is closed.
    #[test]
    fn a_run_is_written_down_unless_the_launch_says_not_to() {
        assert!(with("").resume);
        assert!(!with("--no-resume").resume);
        // And it is its own switch: taking it out leaves the chapters it was built on.
        assert!(with("--no-resume").chapters);
    }

    /// The frame is orb's unless somebody says otherwise at the launch, and the file cannot say
    /// it: a game left running its own frame is a game with the input lag back.
    #[test]
    fn the_frame_is_left_to_the_game_only_when_asked_for_at_the_launch() {
        assert!(with("").own_frame_loop);
        assert!(!with("--no-frame-loop").own_frame_loop);
        assert!(with("--no-frame-loop").frame_hooks);
    }

    /// The compositor's drawing time is swept, so it is given in the microseconds a sweep
    /// steps in, and not given at all when it is to be found while running.
    #[test]
    fn the_compositors_drawing_time_is_pinned_in_microseconds() {
        assert_eq!(with("--compose=200").compose_us, 200);
        assert_eq!(with("").compose_us, 0);
    }

    #[test]
    fn nothing_given_leaves_the_file_alone() {
        let plain = with(r"C:\game\th06.exe");
        assert!(!plain.chapter_tuning && !plain.during_replay && !plain.chapter_stepping);
        assert!(!plain.fast_clear);
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
