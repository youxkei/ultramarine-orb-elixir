//! **The six numbers a run of th06nc carries from one stage to the next, and that a run picked up out of
//! a file comes back with every one of them.**
//!
//! `Th06nc::numbers` reads them where the stage's own build leaves them — the seven fields of
//! `game::th06nc::GAME_MANAGER` its stage-following path at `0x14003af12` does not write, and the seed
//! out of the copy that path's common tail makes at `0x14003b171` — and `Th06nc::put_the_numbers_back`
//! writes them back. None of them is anything the buttons play against: this game's extra lives are
//! bought with score and the panel counts up from whatever the front end left, so a run resumed without
//! them plays out right and is *recorded* wrong, and what a run is for is the number at the end of it.
//!
//! **Read at the stage's own start and nowhere else**, which is what this walks a whole stage for: the
//! numbers a chapter carries are the numbers of the frame the stage it is in began on. The run here
//! therefore has its numbers set in its first stage and is left in the second, whose record is the one
//! picked up.
//!
//! **Two launches and not one**, because the file is the thing under test: a retry inside one launch puts
//! a chapter back out of a snapshot of the game's own memory, where a pick-up reads the numbers out of
//! `resume`'s own record and writes them into a stage the game has just built. Only the second route can
//! lose one.
//!
//! **And the first launch quits with the run still on rather than leaving it**, which is what a launch
//! somebody closes mid-stage is: a run to pick up is a run the process ended during. Leaving would keep
//! the file too — the state `Th06nc::leave_run` asks for builds no screen a run ends at, so
//! `Game::run_finished` answers false for it and nothing discards the slot — and quitting is the one of
//! the two that needs no answer to a question first.
//!
//! **The deaths are not among what this sets**, and that is this game's own doing: `State::deaths` is
//! orb's own count of the player's hit edges — see `game::th06nc::DEATHS`, and why the dword at
//! `0x1404f1e60` is not a run's deaths — so there is no field of the game's to write one into. What a
//! chapter carries of it is what orb counted, and `the_cases_of_th06nc_the_real_game_is_tested_with` is
//! where a death is played for real.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file: Normal and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// **How many frames a launch is given to run its recipe out and quit in**, which is the wall this puts
/// up rather than sitting in a loop: the walk to a run is about five hundred frames of this laid-out
/// game's front end and two stages of it are [`A_STAGE`] each, so twelve thousand is the whole of both
/// launches over again. th06c's own module says sixty thousand and reaches every wait of it early;
/// **this one is the number it is because a wait that runs out has to say so in seconds and not in
/// minutes** — the first two shapes of this test each burned a full bound before naming where the walk
/// stopped.
const FRAMES: u32 = 12_000;

/// How long a stage of this laid-out game lasts here, so that the run walks into the second of them
/// without a test sitting through a whole one.
const A_STAGE: u32 = 600;

/// **The seven numbers, none of them round and none of them a first stage's.** A first stage's are what
/// a build leaves, so a resume that wrote nothing at all would pass against them.
///
/// **The seed is not among them, and that is the game's own doing**: of the nine `resume::Numbers`
/// carries, the seed is the one the stage's build writes itself — out of the copy the common tail of
/// `0x14003af12` makes at `0x14003b171` — so what a stage begins with is that stage's own seed and not
/// the one the stage before it had. What it is in the record for is the stage it is *put back into*, and
/// `Th06nc::seed_the_stage` is what puts it there.
///
/// **And what a point item is worth is one of the seven, which the point items are not.** They are two
/// fields of the manager: `POINT_ITEMS` counts what the run has taken and `POINT_ITEM_VALUE` says what
/// the next one pays, and the second carries across a stage the same way — so a run picked up with the
/// first and not the second scores every point item after it wrong.
const SCORE: u64 = 7_654_320;
const POINT_ITEMS: u16 = 1_820;
const POINT_ITEM_VALUE: u8 = 23;
const POWER: u16 = 96;
const LIVES: i8 = 4;
const BOMBS: i8 = 2;
const EXTRA_LIVES: i8 = 3;

/// The stage the numbers are written in and the stage they are read at — **as `State::stage` counts
/// them, which is from nought**: this game counts its own from one and `Th06nc::read_state` hands the
/// number over less one, the way orb counts stages everywhere.
const WRITTEN_IN: i32 = 0;
const READ_AT: i32 = 1;

/// The line a stage's own start is said with, which is where the numbers of that stage are read — **and
/// that line says the game's own number**, so the second stage is two there and one above.
fn the_stage_begins() -> String {
    format!("resume: stage {} begins with", READ_AT + 1)
}

/// The seven as the game's own memory holds them, read the way the panel and the stage's build read
/// them.
fn the_seven(game: &Fake) -> (u64, u16, u8, u16, i8, i8, i8) {
    let image = game.image();
    let (lives, bombs) = image.lives_and_bombs();
    (
        image.the_score(),
        image.the_point_items(),
        image.what_a_point_item_is_worth(),
        image.the_shots_power(),
        lives,
        bombs,
        image.the_extra_lives_the_score_paid_for(),
    )
}

/// And what they were written as, which is what both launches are held to.
const WRITTEN: (u64, u16, u8, u16, i8, i8, i8) = (
    SCORE,
    POINT_ITEMS,
    POINT_ITEM_VALUE,
    POWER,
    LIVES,
    BOMBS,
    EXTRA_LIVES,
);

/// Frames until `done`, and the launch's own log where it never is — which is what says *where* a walk
/// of this game's front end stopped, the harness's own wait saying only that it did.
fn frames_until(game: &Fake, what: &str, done: impl Fn() -> bool) {
    for _ in 0..FRAMES {
        if done() {
            return;
        }
        game.frame();
    }
    panic!(
        "{what} did not happen in {FRAMES} frame(s):\n  {}",
        game.log()
            .lines()
            .into_iter()
            .filter(|line| !line.contains(" scene=") && !line.contains("memory:"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

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

/// **A run walked into, its numbers written in the first stage, and left in the second — and then picked
/// up, with every one of them back.**
#[test]
fn a_run_of_th06nc_picked_up_comes_back_with_the_numbers_it_was_made_with() {
    in_its_own_process(|| {
        let plays = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 18000
- play_until: {{ frames: 60 }}
  give_up_after: 600
- quit: true
"
        );
        let first = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-the-numbers-written",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(plays.clone());
                config.log_level = LogLevel::Verbose;
            },
        );
        first.stages_last(A_STAGE);
        let log = first.log();
        // The numbers written into the first stage, once it is running: the stage's own build is what
        // leaves them, so they have to be there before the stage after it begins.
        frames_until(&first, "the first stage", || {
            first.state().stage == WRITTEN_IN && first.state().in_run
        });
        let image = first.image();
        image.the_run_has_scored(SCORE, POINT_ITEMS);
        image.a_point_item_is_worth(POINT_ITEM_VALUE);
        image.shot_is_at_power(POWER);
        image.has_lives_and_bombs(LIVES, BOMBS);
        image.the_score_has_paid_for(EXTRA_LIVES);
        frames_until(&first, "the second stage", || log.said(&the_stage_begins()));
        assert_eq!(
            the_seven(&first),
            WRITTEN,
            "the second stage did not begin with the numbers the first was given",
        );
        let dir = first.launch().dir().to_path_buf();
        played_out(&first);
        let left = left_by(&first, &dir);
        assert!(
            left.iter()
                .any(|(path, _)| path.to_string_lossy().contains(SLOT)),
            "the launch left no run at {SLOT}: {:?}\n  what it said:\n    {}",
            left.iter().map(|(path, _)| path).collect::<Vec<_>>(),
            log.lines()
                .into_iter()
                .filter(|line| line.contains("resume:") || line.contains("chapter"))
                .collect::<Vec<_>>()
                .join("\n    ")
        );
        drop(first);

        // And the second launch, which finds that file and is told to pick the run up.
        let picks_up = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 300
- play_until: {{ frames: 600 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let second = Fake::attach_finding(
            Display::agreed(120),
            "th06nc-the-numbers-picked-up",
            &left,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(picks_up.clone());
                config.log_level = LogLevel::Verbose;
            },
        );
        second.stages_last(A_STAGE);
        let log = second.log();
        // **The landing and not the stage number**, which is the wait this wanted: `start_stage` writes
        // that number a frame before the build, so `State::stage` reads the stage asked for while the
        // numbers are still the front end's — a wait on it came back `(0, 0, 0, 2, 3, 0)`, the stock a
        // run starts with. The landing is past every frame the record feeds in, which is past the build
        // the numbers go in at.
        frames_until(&second, "the landing the run is picked up at", || {
            log.said("resume: the landing is the frame that was written down, field for field")
                || log.said("resume: the landing is not the frame written down")
                || log.said("resume: not put back")
        });
        assert_eq!(
            second.state().stage,
            READ_AT,
            "the run was not picked up in the stage its record is for"
        );
        assert_eq!(
            the_seven(&second),
            WRITTEN,
            "the run came back with numbers that are not the ones it was made with:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
