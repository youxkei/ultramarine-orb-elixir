//! **The eight numbers a run of th06c is made with, and that a run picked up out of a file comes back with
//! every one of them.**
//!
//! `Th06c::numbers` reads them where the game's own replay record reads them — `0x14004da71` onwards — and
//! `Th06c::put_the_numbers_back` writes them where its playback does. Eight fields, and none of them is
//! anything the buttons play against: this game's extra lives are bought with score, and the panel counts up
//! from whatever the front end left. So a run resumed without them plays out right and is *recorded* wrong,
//! and what a run is for is the number at the end of it.
//!
//! **Read at the stage's own start and nowhere else**, which is what this walks a whole stage for: the
//! numbers a chapter carries are the numbers of the frame the stage it is in began on, so a number written
//! into a stage already running is a number no chapter of that stage was ever written down with. The run here
//! therefore has its eight set in its first stage and is left in the second, whose record is the one picked
//! up.
//!
//! **And two launches and not one**, because the file is the thing under test: a retry inside one launch puts
//! a chapter back out of a snapshot of the game's own memory, where a pick-up reads eight numbers out of
//! `resume`'s own record and writes them into a stage the game has just built. Only the second route can lose
//! one.
//!
//! **And read out of the game's memory in both**, which is what makes the second launch ask anything at all:
//! the line a pick-up says a stage's start with is built from `Record::start` — the record it is putting
//! back — so it says what was asked for whatever landed. Measured: `put_the_numbers_back` writing
//! `numbers.rank + 1` left that line saying the recorded rank and the run playing on with another, and only
//! a read of the game's own field tells the two apart.
//!
//! **The sub-rank is not among the eight**, and that is the game's own doing rather than an omission:
//! `resume::Numbers` carries the rank, which `0x1400243cf` starts at eight and the enemies read, and the
//! sub-rank at `+0x197c` counts towards the next step of it. A stage's own start puts that counter back to
//! nothing, so there is nothing of it for a chapter to carry.

use std::path::{Path, PathBuf};

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file: Normal, and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// How long a stage of this laid-out game lasts here, so that the run walks into the second of them without a
/// test sitting through a whole one.
const A_STAGE: u32 = 900;

/// **The eight numbers, none of them round and none of them a first stage's.** A first stage's are what a
/// build leaves, so a resume that wrote nothing at all would pass against them.
const SCORE: u32 = 7_654_320;
const POINT_ITEMS: u16 = 1_820;
const POWER: u16 = 96;
const LIVES: i8 = 4;
const BOMBS: i8 = 2;
const RANK: i32 = 23;
const SUB_RANK: i32 = 7;
const POWER_ITEMS: i8 = 41;
const EXTRA_LIVES: i8 = 3;

/// The stage the numbers are written in and the stage they are read at, as this game numbers its own.
const WRITTEN_IN: i32 = 1;
const READ_AT: i32 = 2;

/// The line a stage's own start is said with, which is where the numbers of that stage are read.
fn the_stage_begins() -> String {
    format!("resume: stage {READ_AT} begins with")
}

/// The eight as the game's own memory holds them, read the way the panel and the replay record read them.
fn the_eight(game: &Fake) -> (u32, u16, u16, i8, i8, i32, i8, i8) {
    let image = game.image();
    let (lives, bombs) = image.lives_and_bombs();
    (
        image.the_score(),
        image.the_point_items(),
        image.the_shots_power(),
        lives,
        bombs,
        image.the_rank(),
        image.the_power_items(),
        image.the_extra_lives_the_score_paid_for(),
    )
}

/// And what they were written as, which is what both launches are held to.
const WRITTEN: (u32, u16, u16, i8, i8, i32, i8, i8) = (
    SCORE,
    POINT_ITEMS,
    POWER,
    LIVES,
    BOMBS,
    RANK,
    POWER_ITEMS,
    EXTRA_LIVES,
);

/// A launch played out, the game asked to close by its last step.
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

/// Whether a stage's start has been said at all, which is the frame the numbers of that stage are in place.
fn the_stage_has_begun(game: &Fake) -> bool {
    game.log().said(&the_stage_begins())
}

#[test]
fn a_run_of_th06c_picked_up_comes_back_with_the_numbers_it_was_made_with() {
    in_its_own_process(|| {
        let left = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let first = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-numbers-written",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(left);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        first.stages_last(A_STAGE);

        // The eight written into the run's first stage, which the stage after it begins with: the numbers a
        // chapter carries are read where its stage starts, so this is the one place in the run they can go.
        first.frames_until("the run in its first stage", FRAMES, || {
            first.state().stage + 1 == WRITTEN_IN && first.state().stage_frames > 0
        });
        first.image().the_run_has_scored(SCORE, POINT_ITEMS);
        first.image().shot_is_at_power(POWER);
        first.image().has_lives_and_bombs(LIVES, BOMBS);
        first.image().the_rank_is(RANK, SUB_RANK);
        first.image().the_power_items_towards_the_score(POWER_ITEMS);
        first.image().the_score_has_paid_for(EXTRA_LIVES);

        // The stage after it, read on the frame its start was said on: what a chapter of that stage carries
        // is the numbers of this frame, and the run plays on from here.
        first.frames_until("the second stage begun", FRAMES, || {
            the_stage_has_begun(&first)
        });
        assert_eq!(
            the_eight(&first),
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
        // names: every one of the eight is written where the game's own playback writes it, and the line the
        // stage's start is said with is read off the game's memory after that.
        let picked_up = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ stage: 2, chapter: 2 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let second = Fake::attach_finding(
            Display::agreed(120),
            "th06c-the-numbers-put-back",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(picked_up);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        second.stages_last(A_STAGE);

        // **Read out of the game's own memory and not off the line**, which is the whole point of the second
        // launch: the line a pick-up says a stage's start with is built from the record it is putting back,
        // so it says what was asked for. What landed is what the panel and the replay record would read.
        second.frames_until("the stage the record names built", FRAMES, || {
            the_stage_has_begun(&second)
        });
        assert!(
            second.log().said("resume: from where it stopped"),
            "the run was not picked up where it stopped:\n  {}",
            second.log().lines().join("\n  ")
        );
        assert_eq!(
            the_eight(&second),
            WRITTEN,
            "the run came back with numbers of another run's:\n  {}",
            second.log().lines().join("\n  ")
        );
        played_out(&second);
    });
}
