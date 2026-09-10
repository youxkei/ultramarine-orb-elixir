//! **What a playback of th08 says about itself, frame by frame.**
//!
//! orb drives the updates itself to put a run back: it calls the game's own calc walk again and again inside
//! one frame of its own, feeding a frame's buttons to each, until the stage's clock reaches the frame the
//! chapter began on. What has to hold of that is not visible from the run landing in the right place, and it
//! is `the_updates_a_resume_of_th06c_takes`'s two claims, held here to this game:
//!
//! - **every frame fed says its own line where the log is verbose** — the stage's frame, the generator, the
//!   run's numbers and the player — which is what holds a playback to the run it is reproducing: a run put back
//!   at the right frame with a different world in it is a run put back wrong;
//! - **and a second playback in one session is watched from its own first frame** rather than from where the
//!   one before it stopped.
//!
//! **This game's line opens on `frames=`**, the stage's own count as `Th08::stage_frame` counts it, where
//! th06c's opens on the replay's own frame: see `Th08::reproduction`, whose fields each carry the panel row or
//! the replay header line they were named by.

use std::path::{Path, PathBuf};

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 40_000;

/// How long the laid-out stage lasts: past the table's row at script 790 and into the boss's fight, which is
/// the stage's second and third chapters.
const A_STAGE: u32 = 2_000;

/// Where orb keeps the runs left unfinished, and what it calls one.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-0-ply00a";

/// What opens every line a frame of this game says of itself.
const A_FRAMES_LINE: &str = "played with frames=";

fn configured(config: &mut orb_config::Config, recipe: String) {
    config.autoplay = Some(recipe);
    config.log_level = LogLevel::Verbose;
    config.save_replays = true;
}

fn over_its_own_stage(game: Box<Fake>) -> Box<Fake> {
    game.plays_its_own_stage();
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
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = over_its_own_stage(Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| configured(config, recipe),
    ));
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

/// A launch that finds `files` and plays `recipe`.
fn finding(name: &str, files: &[(PathBuf, Vec<u8>)], recipe: String) -> Box<Fake> {
    over_its_own_stage(Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| configured(config, recipe),
    ))
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

/// The lines the frames of this launch said about themselves, in the order they said them.
fn the_frames_said(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains(A_FRAMES_LINE))
        .collect()
}

/// The walk to a run with the question answered つづきから.
fn picks_it_up() -> String {
    format!(
        "{TO_A_RUN}\
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
"
    )
}

#[test]
fn a_playback_of_th08_says_every_frames_own_line_where_the_log_is_verbose() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th08-playback-lines-left");
        let recipe = format!(
            "{}\
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
",
            picks_it_up()
        );
        let game = finding("th08-playback-lines", &files, recipe);
        played_out(&game);
        let log = game.log();

        assert!(
            log.said("playing its buttons in"),
            "nothing says the run was put back:\n  {}",
            log.lines().join("\n  ")
        );
        let said = the_frames_said(&game);
        assert!(
            !said.is_empty(),
            "a playback said nothing about the frames it fed:\n  {}",
            log.lines().join("\n  ")
        );
        // Every one of them carries the run's own world beside the frame: the generator, the numbers and the
        // player, which is what a run put back at the right frame with a different world in it is caught by.
        for line in &said {
            for named in ["seed=", "stage_seed=", "score=", "lives=", "power=", "at="] {
                assert!(
                    line.contains(named),
                    "a frame of the playback said nothing about {named}: {line}",
                );
            }
        }
        // And the stage's frames they are about climb: a playback feeds the file in the order it was written,
        // so a line whose `frames=` went backwards is a playback out of order.
        let frames: Vec<u32> = said
            .iter()
            .filter_map(|line| {
                let at = line.find(A_FRAMES_LINE)? + A_FRAMES_LINE.len();
                line[at..]
                    .split_whitespace()
                    .next()
                    .and_then(|it| it.parse().ok())
            })
            .collect();
        assert_eq!(
            frames.len(),
            said.len(),
            "a line of the playback did not carry the stage's frame"
        );
        for pair in frames.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "the frames went backwards: {} after {}",
                pair[1],
                pair[0]
            );
        }
    });
}

/// **A second playback in one session is watched from its own first frame**: a run left behind is picked up,
/// played on, given up and picked up again, which is two playbacks of the same slot — the second of them over a
/// file the first run wrote, so a complaint about a jump would be about a frame of the playback before it.
#[test]
fn a_second_playback_of_th08_in_one_session_is_watched_from_its_own_first_frame() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th08-picked-up-twice-left");
        let recipe = format!(
            "{picks}\
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
{picks}\
- wait_until: {{ frames: 300 }}
- quit: true
",
            picks = picks_it_up()
        );
        let game = finding("th08-picked-up-twice", &files, recipe);
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
