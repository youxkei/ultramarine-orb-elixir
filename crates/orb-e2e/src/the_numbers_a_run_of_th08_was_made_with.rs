//! **The numbers a run of th08 is made with, and that a run picked up out of a file comes back with them.**
//!
//! `Th08::numbers` reads them where the game's own replay record reads them and `Th08::put_the_numbers_back`
//! writes them where its playback does — see `resume::Numbers` for the fields. **The score, the power, the
//! lives, the bombs and the point items are the ones asked for here**, the five the laid-out game's memory
//! carries where this test can write them and read them back.
//!
//! **Read at the stage's own start and nowhere else**, which is what this walks a whole stage for: the
//! numbers a chapter carries are the numbers of the frame the stage it is in began on. So the run has them
//! set in its first stage and is left in the second, whose record is the one picked up — **and that second
//! stage's score is the panel's copy**, the build having handed the score to it and put the score to nought
//! (see `a_stage_transition_of_th08`), which is the number a pick-up has to put back.
//!
//! **And two launches and not one**, for `the_numbers_a_run_of_th06c_was_made_with`'s reason: a retry puts a
//! chapter back out of a snapshot of the game's memory, where a pick-up reads the numbers out of `resume`'s own
//! record and writes them into a stage the game has just built. Only the second route can lose one.

use std::path::{Path, PathBuf};

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-0-ply00a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// How long a stage of this laid-out game lasts here, so that the run walks into the second of them without a
/// test sitting through a whole one.
const A_STAGE: u32 = 1_500;

/// **The numbers, none of them round and none of them a first stage's.** A first stage's are what a build
/// leaves, so a resume that wrote nothing at all would pass against them.
const SCORE: u32 = 7_654_320;
const POINT_ITEMS: u32 = 182;
const POWER: f32 = 72.0;
const LIVES: f32 = 4.0;
const BOMBS: f32 = 1.0;

/// The line the second stage's own start is said with, which is where the numbers of that stage are read.
const THE_SECOND_STAGE_BEGINS: &str = "resume: stage 2 begins with";

/// The numbers as the game's own memory holds them: the score the panel draws, the point items, the power,
/// the lives and the bombs.
fn the_numbers(game: &Fake) -> (u32, u32, u16, i8, i8) {
    let state = game.state();
    let (_, shown) = game.image().the_score_and_the_panels_copy();
    (
        shown,
        game.image().the_point_items(),
        state.power,
        state.lives,
        state.bombs,
    )
}

/// And what they were written as, which is what both launches are held to.
const WRITTEN: (u32, u32, u16, i8, i8) =
    (SCORE, POINT_ITEMS, POWER as u16, LIVES as i8, BOMBS as i8);

/// A launch played out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the recipe never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  ")
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
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

#[test]
fn a_run_of_th08_picked_up_comes_back_with_the_numbers_it_was_made_with() {
    in_its_own_process(|| {
        let first = launched(
            "th08-the-numbers-written",
            format!(
                "{TO_A_RUN}\
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
            ),
        );

        // The numbers written into the run's first stage, which the stage after it begins with: the numbers a
        // chapter carries are read where its stage starts, so this is the one place in the run they can go.
        first.frames_until("the run in its first stage", FRAMES, || {
            let state = first.state();
            state.in_run && state.stage == 0 && state.stage_frames > 0
        });
        first.image().the_run_has_scored(SCORE, POINT_ITEMS);
        first.image().shot_is_at_power(POWER);
        first.image().has_lives_and_bombs(LIVES, BOMBS);

        first.frames_until("the second stage begun", FRAMES, || {
            first.log().said(THE_SECOND_STAGE_BEGINS)
        });
        assert_eq!(
            the_numbers(&first),
            WRITTEN,
            "the run's second stage did not begin with the numbers written into its first",
        );
        played_out(&first);
        let files = left_by(&first, first.launch().dir());
        assert!(
            files.iter().any(|(inside, _)| {
                inside.to_string_lossy().contains(SLOT)
                    && inside.to_string_lossy().contains("pointdevice_resume")
            }),
            "no chapter file of {SLOT} was left: {:?}",
            files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
        );
        drop(first);

        // And the second launch, which finds that file, answers つづきから and builds the stage the record
        // names: every one of the numbers is written where the game's own playback writes it, and read back
        // off the game's memory once the run has landed.
        let second = launched(
            "th08-the-numbers-picked-up",
            format!(
                "{TO_A_RUN}\
- answer: {{ question: resume, item: 0 }}
  give_up_after: 1800
- wait_until: {{ frames: 600 }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
            ),
        );
        for (inside, bytes) in &files {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        let log = second.log();
        second.frames_until("the run landed", FRAMES, || {
            log.said("resume: the landing is")
        });
        assert!(
            log.said("the run is being built at that stage"),
            "the run was not picked up in the stage it was left in:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            the_numbers(&second),
            WRITTEN,
            "the run picked up did not come back with the numbers it was made with:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        played_out(&second);
    });
}
