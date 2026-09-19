//! **What a playback of th06nc says about itself, frame by frame.**
//!
//! orb drives the updates itself to put a run back: it calls the game's own calc walk again and again
//! inside one frame of its own, feeding a frame's buttons to each, until the stage's clock reaches the
//! frame the chapter began on. What has to hold of that is not visible from the run landing in the right
//! place.
//!
//! **Every frame fed says its own line where the log is verbose** — the frame, the frame of the file it
//! came from, the word, the player, the generator and the run's numbers. Which is what holds a playback to
//! the run it is reproducing: a run put back at the right frame with a different world in it is a run put
//! back wrong, and the generator alone says only that one had.
//!
//! **And this game's own line is two names short of th06c's**, which is what these read for rather than
//! th06c's list: `Th06nc::fingerprint` answers eight numbers and not ten — there is no replay manager's
//! frame here and no rank, each for a reading this game has not got — so neither `replay_frame=` nor
//! `rank=` is a name a line of this game carries, and the frame of the file is the number the line opens
//! with.
//!
//! **The waits are frames and not chapters**, which is every module over a run of this game: its
//! `chapters.rs` holds seven rows with nothing in them, so which chapter a run is in past its first is a
//! thing of what the stage's own waves did rather than of a table.
//!
//! **And th06c's second claim beside it is not here**, which is a gap this file names rather than one it
//! hides: that a second playback in one session is watched from its own first frame. A recipe that leaves
//! the run and walks the front end again reaches `resume: … the run is being built at that stage` a second
//! time and the laid-out game never builds it — no `the stage seeded from` line follows — so what such a
//! test would assert is the fake's own hole. This change's own `TODO:` carries it, beside the pick-up
//! defect a launch of the real game found in the same place.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long a stage of this laid-out game lasts, so that a run left mid-stage is left mid-stage.
const A_STAGE: u32 = 6_000;

/// Where orb keeps the runs left unfinished, and what it calls one.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file: Normal and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// **How far into the stage the run is played before it is left**, which is what the playback then has to
/// feed back: far enough that a playback is many updates and not one.
const PLAYED_FOR: u32 = 600;

/// The walk from the title to a run of this game with the run started, which is what leaves the run this
/// file picks up.
fn to_a_run_started() -> String {
    format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
"
    )
}

/// And the walk that finds orb's question about picking a run up, and answers it with where it stopped.
fn to_the_run_picked_up() -> String {
    format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 3600
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
"
    )
}

fn launched(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.stages_last(A_STAGE);
    game
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
        game.log()
            .lines()
            .into_iter()
            .filter(|line| !line.contains(" scene=") && !line.contains("resume: frame "))
            .collect::<Vec<_>>()
            .join("\n  ")
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

/// A run played a while into its first stage and left there, and the files that launch left behind.
fn a_run_left_behind(name: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{}\
- play_until: {{ frames: {PLAYED_FOR} }}
  give_up_after: 6000
- quit: true
",
        to_a_run_started()
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

/// The lines a playback said about the frames it fed, in the order it said them.
///
/// **Matched on the frame of the *file* and not on `replay_frame=`, which is th06c's own**: that name is
/// the game's own replay manager's count of frames and this game has no reading for one —
/// `Th06nc::fingerprint`'s doc carries why — so a line of this game opens `resume: frame N played`.
fn the_frames_fed(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains(FED) && line.contains(" played"))
        .collect()
}

/// What a line about one frame of the playback opens with, the frame of the file coming straight after it.
const FED: &str = "resume: frame ";

#[test]
fn a_playback_of_th06nc_says_every_frames_own_line_where_the_log_is_verbose() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-playback-lines-left");
        let recipe = format!(
            "{}\
- play_until: {{ frames: 300 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
",
            to_the_run_picked_up()
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06nc-playback-lines",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        game.stages_last(A_STAGE);
        played_out(&game);
        let log = game.log();

        let fed = the_frames_fed(&game);
        assert!(
            !fed.is_empty(),
            "a playback said nothing about the frames it fed:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| !line.contains(" scene="))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        // Every one of them carries the run's own world beside the frame: the generator and the numbers,
        // which is what a run put back at the right frame with a different world in it is caught by.
        for line in &fed {
            for named in ["player=", "randoms=", "rng=", "score=", "extras="] {
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
                let at = line.find(FED)? + FED.len();
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
            // **And none of them is skipped, which is the half that says a fed word reaches an update.**
            // `resume::Record::feed` answers the same word for the same frame twice on purpose — the game
            // reads the keyboard more than once on a frame no update follows — but a frame *stepped over*
            // is a word fed to nothing, and the update that would have acted on it acts on the frame
            // before's instead. Which is what makes a run played with a press missing look like a run
            // that simply went differently, and what the frame the record is keyed by has to be held to.
            assert!(
                pair[1] <= pair[0] + 1,
                "the playback stepped over a frame of the file: {} after {}",
                pair[1],
                pair[0]
            );
        }
        // **Said from orb's own side too**, the report being the one thing that catches a skip without a
        // run to hold the playback against: `Record::feed` writes it once for a playback that jumps.
        assert!(
            !log.said("resume: the playback was fed frame"),
            "orb reported the playback skipping a frame of the run: {:?}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("the playback was fed frame"))
                .collect::<Vec<_>>()
        );
        // The run landed, which is what says the lines above are a playback and not a run starting over.
        assert!(
            log.said("resume: the landing is") || log.said("playing its buttons in"),
            "nothing says the run was put back",
        );
    });
}
