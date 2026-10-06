//! **The file a chapter of th11 is written down in is written from a thread of orb's own, not in the frame.**
//!
//! One is written every time a chapter begins — see `resume.rs`, and *Picking a run up again*. A `WriteFile`
//! takes what it takes, and a write in the frame's own thread is a write the frame waits for: which is
//! `the_run_of_th10_written_down_off_the_frame`'s claim, held here to this game. What a test can say about
//! *where* a write happened is the thread it came from, which the simulated Windows records per file: the
//! frames run on the test's own thread, so a write from any other thread is one that is not in the frame.

use std::path::PathBuf;
use std::time::Duration;

use crate::fake::th11::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long the laid-out stage lasts: past its midboss's arrival at frame 1200, whose first attack is the
/// stage's second chapter.
const A_STAGE: u32 = 4_500;

/// Where orb keeps the runs left unfinished, and what it calls one — written out rather than taken from
/// `resume`, where both are private.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file: Normal, and Reimu with her first partner.
const SLOT: &str = "normal-reimua";

/// A disk that takes longer over this file than a frame has, in microseconds. A frame at sixty is 16,667us.
const TAKES: u64 = 40_000;

/// The recipe that walks the title into a run, plays it into its stage's second chapter and stops there, so
/// that two chapters have been written by the time anything is asserted.
fn to_a_chapter() -> String {
    format!(
        "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- wait_until: {{ frames: 120 }}
- quit: true
"
    )
}

fn launched(name: &str, level: LogLevel) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(to_a_chapter());
            config.log_level = level;
            config.save_replays = true;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    // Firing nothing, a run's dying being none of this file's subject.
    game.fires_at_the_player(false);
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

/// Where the run these play is written down.
fn written_down_in(game: &Fake) -> PathBuf {
    game.launch()
        .dir()
        .join(KEPT_IN)
        .join(format!("{SLOT}.{KEPT_AS}"))
}

#[test]
fn the_chapter_of_th11_written_down_is_written_from_a_thread_of_orbs_own() {
    in_its_own_process(|| {
        // `verbose`, because the line saying the file was written is one of those: it is written once a
        // chapter, and this is what says which thread wrote it and how long the write took.
        let game = launched("th11-written-off-the-frame", LogLevel::Verbose);
        played_out(&game);
        let log = game.log();

        let path = written_down_in(&game);
        let wrote = game
            .sim()
            .files()
            .written_from(&path)
            .expect("the chapter was written down");
        assert_ne!(
            wrote,
            orb_api::thread::current_id(),
            "the file was written in the frame's own thread, which is where a slow disk stops the game",
        );
        assert!(
            log.said(&format!("written to {}", path.display())),
            "the chapter was written and the log does not say where:\n  {}",
            log.lines().join("\n  ")
        );
        // What it holds is the chapter and not a file the writer left half written: the run is among the ones
        // a later launch would offer.
        assert_eq!(
            orb_core::resume::left(game.launch().dir()),
            vec![SLOT.to_owned()],
            "the run is not among the ones left unfinished",
        );
    });
}

#[test]
fn a_write_of_a_chapter_of_th11_that_took_longer_than_a_frame_is_said_at_the_ordinary_level() {
    in_its_own_process(|| {
        // `normal`, which is what a run is logged at when nobody is looking into anything: a disk this slow
        // is worth saying to somebody who did not ask for detail.
        let game = launched("th11-written-slowly", LogLevel::Normal);
        game.sim()
            .files()
            .writes_slowly(written_down_in(&game), Duration::from_micros(TAKES));
        played_out(&game);
        let log = game.log();

        let said = log
            .lines()
            .into_iter()
            .find(|line| line.contains("resume: writing"))
            .unwrap_or_else(|| {
                panic!(
                    "a write that took longer than a frame was not said at the ordinary level:\n  {}",
                    log.lines().join("\n  ")
                )
            });
        assert!(
            said.contains("took longer than a frame"),
            "the line does not say what is wrong with it: {said}",
        );
        let us: u64 = said
            .rsplit(' ')
            .next()
            .and_then(|last| last.strip_suffix("us"))
            .and_then(|us| us.parse().ok())
            .unwrap_or_else(|| panic!("the line does not end in the microseconds it took: {said}"));
        assert!(
            us >= TAKES,
            "the line said {us}us for a write declared to take {TAKES}us",
        );
        assert!(
            log.said("stage 1 chapter 2"),
            "a run whose write was slow did not reach the chapter it was playing for:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
