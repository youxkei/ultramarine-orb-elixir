//! **What a chapter of th08 retried leaves in the record of a run, and whether a playback of that record is
//! the run.**
//!
//! What `the_run_a_retried_chapter_of_th07_writes_down.rs` asks of that game, asked of this one. A retry is not
//! a thing the record knows about: the buttons are one entry per stage frame, and an attempt that is rewound
//! plays the same frames again and writes over them — so what the file holds is the frames of the attempt that
//! survived, stitched onto the frames before the chapter it was retried at.
//!
//! **But the surviving attempt was not played from the stage's start.** It was played from a snapshot put back
//! at the chapter's own frame, where a playback of the file reaches that frame by playing the buttons under it
//! instead. The two are the same state only as far as the restore puts everything back — and for this game
//! that is twenty-one megabytes of `.data` and five heap regions with the sound manager's stream and every
//! `.anm` slot's sheets left out, the run's seal among what comes back — so whatever the restore leaves
//! standing from the attempt that was lost is state a playback has never seen, and from there the same
//! buttons are a different run.
//!
//! **Two launches, the way a person meets it**: the first dies in the stage's second chapter, takes it again
//! and plays on into the third, where it is left; the second picks it up. Both are played by recipes over the
//! laid-out stage — the pick-up tests of `a_pointdevice_run_of_th08.rs` are the same pair without the retry.

use std::path::PathBuf;

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage of the laid-out game lasts: past stage 1's one row of the table and on to its boss, who
/// arrives for the stage's last 900 frames and is the chapter after the row — the laid-out stage having no
/// midboss.
const A_STAGE: u32 = 4_500;

/// **The run that dies in its second chapter and takes it again**, `a_death_answered_by_the_chapter_again`'s
/// walk and hit, played on into the third chapter after the retry and left there rather than played for a
/// bound of frames.
const RETRIED_AND_LEFT: &str = "
- wait_until: { screen: 0 }
  give_up_after: 1800
- press: [up]
  frames: 30
  until: { cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 4 }
  give_up_after: 1800
- press: [decide]
  frames: 30
  until: { screen: 5 }
  give_up_after: 900
- press: [decide]
  frames: 12
- wait_until: { frames: 120 }
- press: [decide]
  frames: 12
- wait_until: { frames: 300 }
- play_until: { stage: 1, chapter: 2 }
  give_up_after: 3600
- take_hits: { chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }
- play_until: { stage: 1, chapter: 3 }
  give_up_after: 18000
- leave: true
- wait_until: { frames: 300 }
- quit: true
";

/// A launch of this game played by `recipe`, over a stage of its own.
fn launched(name: &str, recipe: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.self_check = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

/// **Played until its recipe quits it**, and failed where it never does.
fn runs_out(game: &Fake) {
    const FRAMES: u32 = 40_000;
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the launch never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  "),
    );
}

/// What a launch said of the run it left and the run it picked up, without the line a frame is.
fn what_it_said_of_the_run(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| {
            (line.contains("resume: ") && !line.contains("resume: frame "))
                || line.contains("retry")
                || line.contains("died in")
        })
        .collect()
}

/// A chapter retried, and the run picked up out of the file the chapter after it left.
#[test]
fn a_run_of_th08_whose_chapter_was_retried_is_put_back_at_the_chapter_after_it() {
    in_its_own_process(|| {
        let first = launched("th08-retried-then-left", RETRIED_AND_LEFT);
        runs_out(&first);
        let log = first.log();
        let said = what_it_said_of_the_run(&first);
        assert!(
            log.said("retry chapter 2 (retry 1)"),
            "the second chapter was not taken again:\n  {}",
            said.join("\n  "),
        );
        // The chapter after it, which is the file this run leaves: its buttons are the frames before the
        // retried chapter and the frames of the attempt that survived, stitched together.
        assert!(
            log.said("resume: stage 1 chapter 3 ("),
            "the chapter after the retry was not written down:\n  {}",
            said.join("\n  "),
        );
        assert!(
            !log.lines()
                .iter()
                .any(|line| line.contains("self_check: ") && !line.contains(" 0 saved region(s)")),
            "a snapshot of the run did not give back the state it was taken from:\n  {}",
            said.join("\n  "),
        );
        let dir = first.dir().to_path_buf();
        let files = first.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                Some((
                    path.strip_prefix(&dir).ok()?.to_path_buf(),
                    files.get(&path)?,
                ))
            })
            .collect();

        // Picked up, which is where a stitched record tells: the playback reaches the retried chapter's frame
        // by playing the buttons under it, where the run reached it by having its memory put back.
        let second = launched(
            "th08-retried-then-picked-up",
            include_str!("../../orb-real/recipes/th08/the_run_picked_up_where_it_was_left.yaml"),
        );
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let said = what_it_said_of_the_run(&second);
        assert!(
            second
                .log()
                .said("resume: the landing is the frame that was written down, field for field"),
            "the run whose chapter was retried did not land where the file said:\n  {}",
            said.join("\n  "),
        );
    });
}
