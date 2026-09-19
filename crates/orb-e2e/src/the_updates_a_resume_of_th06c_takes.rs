//! **What a playback of th06c says about itself, frame by frame.**
//!
//! orb drives the updates itself to put a run back: it calls the game's own calc walk again and again inside
//! one frame of its own, feeding a frame's buttons to each, until the stage's clock reaches the frame the
//! chapter began on. What has to hold of that is not visible from the run landing in the right place.
//!
//! **Every frame fed says its own line where the log is verbose** — the frame, the frame of the file it came
//! from, the word, the player, the generator and the run's numbers. Which is what holds a playback to the run
//! it is reproducing: a run put back at the right frame with a different world in it is a run put back wrong,
//! and the generator alone says only that one had. These lines cost log and nothing else, which is why they
//! are at that level rather than behind the chapter fingerprinting that took a boundary's snapshot from 2ms to
//! 204ms.
//!
//! **And th07's second claim beside it**, that a second playback in one session is watched from its own first
//! frame rather than from where the one before it stopped. Two playbacks in one session are a run given up and
//! picked up again without the process ending, which on this game is the front end walked twice: a recipe
//! whose second walk started at the difficulty select gave up, the front end a leave rebuilds coming up on
//! the logo — so the walk is `TO_A_RUN` again, from that logo, with the mode question answered a second time
//! the way a person pressing at the title is asked it again.

use std::path::{Path, PathBuf};

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// Where orb keeps the runs left unfinished, and what it calls one.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-reimu-a";

fn launched(name: &str, recipe: String) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// What a launch left under orb's directory, re-rooted for the next.
fn left_by(game: &Fake, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            let inside = path.strip_prefix(dir).ok()?.to_path_buf();
            Some((inside, files.get(&path)?))
        })
        .collect()
}

/// A run played into its stage's second chapter and left there, and the files that launch left behind.
fn a_run_left_behind(name: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = launched(name, recipe);
    played_out(&game);
    let files = left_by(&game, game.launch().dir());
    assert!(
        files.iter().any(|(inside, _)| {
            let said = inside.to_string_lossy();
            said.contains(KEPT_IN) && said.contains(SLOT) && said.ends_with(KEPT_AS)
        }),
        "no chapter file of {SLOT} was left: {:?}",
        files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
    );
    files
}

/// How many runs of this launch landed where their file said, field for field — which is how many playbacks
/// it made.
fn landings(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("resume: the landing is the frame that was written down, field for field")
        })
        .count()
}

/// The lines a playback said about the frames it fed, in the order it said them.
fn the_frames_fed(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains("played with replay_frame="))
        .collect()
}

#[test]
fn a_playback_of_th06c_says_every_frames_own_line_where_the_log_is_verbose() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06c-playback-lines-left");
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06c-playback-lines",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        played_out(&game);
        let log = game.log();

        let fed = the_frames_fed(&game);
        assert!(
            !fed.is_empty(),
            "a playback said nothing about the frames it fed:\n  {}",
            log.lines().join("\n  ")
        );
        // Every one of them carries the run's own world beside the frame: the generator and the numbers, which
        // is what a run put back at the right frame with a different world in it is caught by.
        for line in &fed {
            for named in ["player=", "randoms=", "rng=", "score=", "extras=", "rank="] {
                assert!(
                    line.contains(named),
                    "a frame of the playback said nothing about {named}: {line}",
                );
            }
        }
        // And the frames of the file they came from climb: a playback feeds the file in the order it was
        // written, so a line whose `replay_frame` went backwards is a playback out of order.
        let of_the_file: Vec<i64> = fed
            .iter()
            .filter_map(|line| {
                let at = line.find("replay_frame=")? + "replay_frame=".len();
                line[at..]
                    .split_whitespace()
                    .next()
                    .and_then(|it| it.parse().ok())
            })
            .collect();
        assert_eq!(
            of_the_file.len(),
            fed.len(),
            "a line of the playback did not carry a frame of the file"
        );
        for pair in of_the_file.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "the playback fed the file out of order: {} after {}",
                pair[1],
                pair[0]
            );
        }
        // The run landed, which is what says the lines above are a playback and not a run starting over.
        assert!(
            log.said("resume: the landing is") || log.said("playing its buttons in"),
            "nothing says the run was put back:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A playback of th06c puts its run back inside the one frame of the host it began on**, nothing of the run
/// being played forward drawn: every record the trace takes of it carries that frame's own number. It was
/// three hundred updates to a frame of the host once, each frame drawing, which on the real game was the run
/// fast-forwarded on the screen at two frames a second — and a pick-up is asked to be back at once and draw
/// nothing on the way.
#[test]
fn a_playback_of_th06c_puts_its_run_back_inside_one_frame_of_the_host() {
    in_its_own_process(|| {
        const THE_OLD_UPDATES_A_FRAME: u32 = 300;
        let files = a_run_left_behind("th06c-playback-one-frame-left");
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ frames: 60 }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06c-playback-one-frame",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        played_out(&game);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let log = game.log();
        let landed_at = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("resume: stage 1 chapter 2 ")?;
                let (_, after) = after.split_once(" at frame ")?;
                after.split(',').next()?.parse::<u32>().ok()
            })
            .expect("the log says where the chapter the run was left in begins");
        assert!(
            landed_at > THE_OLD_UPDATES_A_FRAME,
            "the chapter the run was left in begins at frame {landed_at}, which a playback never had to go \
             past one frame of the host to reach",
        );
        let played_back: std::collections::BTreeSet<u32> = records
            .iter()
            .filter(|record| {
                record.state.is_some_and(|state| {
                    state.playing && state.stage_frames > 1 && state.stage_frames < landed_at
                })
            })
            .map(|record| record.frame)
            .collect();
        assert_eq!(
            played_back.len(),
            1,
            "the playback went on over the host's frames {played_back:?}:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// The records of a launch's trace, once it has closed.
fn the_records_of(game: &Fake) -> Vec<orb_core::trace::Record> {
    orb_core::trace::close();
    let bytes = game
        .sim()
        .files()
        .get(game.dir().join(orb_real::TRACE_FILE))
        .expect("the launch wrote a trace");
    orb_core::trace::read(&bytes).expect("the trace reads back")
}

/// The word each stage frame of a run was first played with, from `from` on — the records of the frames the
/// rules were asked for, which carry where the player was.
fn the_words_from(records: &[orb_core::trace::Record], from: u32) -> Vec<(u32, u16)> {
    let mut seen = std::collections::BTreeMap::new();
    for record in records {
        if let Some(state) = record.state
            && state.in_run
            && state.stage_frames >= from
            && record.at.is_some()
        {
            seen.entry(state.stage_frames).or_insert(record.word);
        }
    }
    seen.into_iter().collect()
}

/// **A run picked up is played on by the rules the way the run it was cut from was**, frame for frame from the
/// landing: what the rules do on a frame depends on the frames before it, and a playback that showed them none
/// had them go on from what they last saw before it — a pick-up of the real Extra's chapter 7 landed field for
/// field and was pressed down on the next frame where the run had been pressed up and right.
#[test]
fn a_run_of_th06c_picked_up_is_played_on_as_the_run_it_was_cut_from_was() {
    in_its_own_process(|| {
        const PLAYED_ON: u32 = 600;
        let traced = |config: &mut orb_config::Config| {
            config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
        };
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 6000
- play_until: {{ frames: {} }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
",
            PLAYED_ON * 2
        );
        let run = Fake::attach_configured(
            Display::agreed(120),
            "th06c-played-on-as-cut-left",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
                traced(config);
            },
        );
        played_out(&run);
        let files: Vec<_> = left_by(&run, run.launch().dir())
            .into_iter()
            .filter(|(inside, _)| inside != Path::new(orb_real::TRACE_FILE))
            .collect();
        let the_run = the_records_of(&run);
        drop(run);

        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ frames: {PLAYED_ON} }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06c-played-on-as-cut",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
                traced(config);
            },
        );
        played_out(&game);
        let log = game.log();
        let landed_at = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("resume: stage 1 chapter ")?;
                let (_, after) = after.split_once(" at frame ")?;
                after.split(',').next()?.parse::<u32>().ok()
            })
            .expect("the log says where the chapter the run was left in begins");
        let picked_up = the_words_from(&the_records_of(&game), landed_at + 1);
        let the_run = the_words_from(&the_run, landed_at + 1);
        let both = picked_up.len().min(the_run.len());
        assert!(
            both > PLAYED_ON as usize / 2,
            "the two runs have {both} frames after the landing at {landed_at} to hold against each other",
        );
        assert_eq!(
            picked_up[..both],
            the_run[..both],
            "the run picked up at frame {landed_at} was played otherwise than the run it was cut from:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A second playback in one session is watched from its own first frame.**
///
/// What watches one is kept beside the buttons: the frame last fed, so that a jump in them is said, and
/// whether this playback has already been found to have parted. Both are a playback's own, and a run given up
/// and picked up again is a second playback — one that starts at frame 0 again, so the frame last fed by the
/// one before it is a jump backwards that nothing jumped.
///
/// **Two pick-ups in one launch, walked twice through the front end**: a run left behind is picked up, played
/// on, given up and picked up again, which is two playbacks of the same file — the second of them over a file
/// the first run wrote, so the frames it feeds are its own and the complaint would be about a frame of the
/// playback before it.
#[test]
fn a_second_playback_of_th06c_in_one_session_is_watched_from_its_own_first_frame() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06c-picked-up-twice-left");
        let picks_it_up = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
"
        );
        let recipe = format!(
            "{picks_it_up}\
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
{picks_it_up}\
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06c-picked-up-twice",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        played_out(&game);
        let log = game.log();

        assert_eq!(
            landings(&game),
            2,
            "one launch did not put the run back twice:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("resume: the playback was fed frame"),
            "a playback was held against the frames of the one before it:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
