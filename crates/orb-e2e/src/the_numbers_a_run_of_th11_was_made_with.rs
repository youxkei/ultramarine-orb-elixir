//! **The numbers a run of th11 is made with, and that a run picked up out of a file comes back with them.**
//!
//! `Th11::numbers` reads them where the game's own replay record reads them and `Th11::put_the_numbers_back`
//! writes them where its playback does — see `resume::Numbers` for the fields. **The score, the power, the lives,
//! the point items' value, the number at `0x4a5744`, the continues and the extends are the ones asked for here**,
//! the seven of the run's numbers a stage of the laid-out game neither steps nor starts again but for the score its
//! end adds, so what this writes into one is what the next begins with, the stage won on top of the score. An
//! extend dropped is one the score hands out a second time.
//!
//! **Read at the stage's own start and nowhere else**, which is what this walks a whole stage for: the numbers a
//! chapter carries are the numbers of the frame the stage it is in began on. So the run has them set in its first
//! stage and is left in the second, whose record is the one picked up — the stage after one won being built with
//! the numbers carried (scene 12 leaving `+0x564` at nought, 0x42b279, which the build's 0x41f932 tests) rather than
//! the defaults a fresh run's build writes.
//!
//! **And two launches and not one**, for `the_numbers_a_run_of_th08_was_made_with`'s reason: a retry puts a
//! chapter back out of a snapshot of the game's memory, where a pick-up reads the numbers out of `resume`'s own
//! record and writes them into a stage the game has just built. Only the second route can lose one.

use std::path::{Path, PathBuf};

use crate::fake::th11::{A_STAGE_WON_SCORES, Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-reimua";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// How long a stage of this laid-out game lasts here, so that the run walks into the second of them without a
/// test sitting through a long one: too short for a midboss, and the boss for its last 900 frames.
const A_STAGE: u32 = 1_500;

/// **The numbers, none of them round and none of them a first stage's.** A first stage's are what a build
/// leaves, so a resume that wrote nothing at all would pass against them.
const SCORE: u32 = 765_432;
const POWER: i32 = 72;
const LIVES: i32 = 4;
const POINT_VALUE: i32 = 23_450;
const THE_NUMBER_AT_4A5744: i32 = 3;
const CONTINUES: i32 = 2;
const EXTENDS: i32 = 5;
/// **And where the small items' ring stands**, which no build starts anywhere but nought and a run carries out
/// of every stage before: what a small item does is made of the slot it goes in.
const THE_SMALL_ITEMS_RING_AT: i32 = 1_234;

/// The line the second stage's own start is said with, which is where the numbers of that stage are read.
const THE_SECOND_STAGE_BEGINS: &str = "resume: stage 2 begins with";

/// The numbers as the game's own memory holds them.
fn the_numbers(game: &Fake) -> (u32, i32, i32, i32, i32, i32, i32) {
    let (score, power, lives) = game.image().the_runs_numbers();
    let (point_value, number) = game.image().the_point_value_and_the_number_at_4a5744();
    let (continues, extends) = game.image().the_continues_and_the_extends();
    (score, power, lives, point_value, number, continues, extends)
}

/// And what they were written as, the first stage won on top of the score, which is what both launches are held
/// to.
const WRITTEN: (u32, i32, i32, i32, i32, i32, i32) = (
    SCORE + A_STAGE_WON_SCORES,
    POWER,
    LIVES,
    POINT_VALUE,
    THE_NUMBER_AT_4A5744,
    CONTINUES,
    EXTENDS,
);

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
    game.fires_at_the_player(false);
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

#[test]
fn a_run_of_th11_picked_up_comes_back_with_the_numbers_it_was_made_with() {
    in_its_own_process(|| {
        let first = launched(
            "th11-the-numbers-written",
            format!(
                "{TO_THE_SHOT}{TO_A_RUN}\
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
        first.image().the_run_has(SCORE, POWER, LIVES);
        first
            .image()
            .the_run_also_has(POINT_VALUE, 0, THE_NUMBER_AT_4A5744, CONTINUES, EXTENDS, 0);
        first
            .image()
            .the_small_items_ring_is_at(THE_SMALL_ITEMS_RING_AT);

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
            "th11-the-numbers-picked-up",
            format!(
                "{TO_THE_SHOT}\
- press: [decide]
  frames: 10
  until: {{ question: resume }}
  give_up_after: 900
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
        assert_eq!(
            second.image().where_the_small_items_ring_is(),
            THE_SMALL_ITEMS_RING_AT,
            "the run picked up did not come back with the small items' ring where its stage began with it",
        );
        played_out(&second);
    });
}
