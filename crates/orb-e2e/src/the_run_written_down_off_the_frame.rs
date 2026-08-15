//! **The file a chapter is written down in is written from a thread of orb's own, not in the frame.**
//!
//! One is written every time a chapter begins — see `resume.rs`, and *Picking a run up again* — which is
//! a few tens of kilobytes at every boundary. A `WriteFile` takes what it takes: on one machine the
//! boundaries stopped the game for about a second each, and a write in the frame's own thread is a write
//! the frame waits for, whatever the disk, the antivirus watching it or the folder being synced decide
//! to do about it.
//!
//! So the frame reads the game and hands the chapter over, and a thread of orb's does the encoding and
//! the write. What a test can say about *where* a write happened is the thread it came from, which the
//! simulated Windows records per file: the frames run on the test's own thread, so a write from any
//! other thread is one that is not in the frame.

use crate::fake::th06::{CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// The file this run's chapter is written down in, under the directory `resume.rs` keeps them in.
fn written_down_in(game: &Fake) -> std::path::PathBuf {
    game.dir()
        .join("pointdevice_resume")
        .join("normal-reimu-a.msgpack")
}

/// 被弾 and then タイトルに戻る, which is the way out that leaves the chapter behind for the next launch
/// to offer: up once from the item the cursor starts on, and its question answered はい.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.hit();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
}

/// A write that took longer than a frame is said at the level a run is ordinarily logged at, not the
/// one somebody has to know to ask for.
///
/// **This is the line the whole arrangement is for.** A session on a machine where the boundaries stopped
/// the game for about a second was played through with the write off the frame and nothing in the log
/// said what the write had cost — it was a `verbose` line, and nobody knew to ask for `verbose`. The
/// evidence for a slow disk has to be in the log the run already writes.
#[test]
fn a_write_that_took_longer_than_a_frame_is_said_at_the_ordinary_level() {
    in_its_own_process(|| {
        // `normal`, which is what a run is logged at when nobody is looking into anything.
        let game = Fake::attach("written-slowly", the_run(), |config| {
            config.log_level = LogLevel::Normal;
        });
        let log = game.log();
        // A disk that takes longer over this file than a frame has: what the machine this is about does.
        game.sim()
            .files()
            .writes_slowly(written_down_in(&game), TAKES);
        game.in_a_pointdevice_run();

        game.frames_until_a_thread("the slow write said out loud", || {
            log.said("resume: writing")
        });
        let said = log
            .lines()
            .into_iter()
            .find(|line| line.contains("resume: writing"))
            .expect("the slow write was said");
        assert!(
            said.contains("took longer than a frame"),
            "the line does not say what is wrong with it: {said}",
        );
        // With the microseconds in it, since what the line is for is saying how bad the disk is.
        let us: u64 = said
            .rsplit(' ')
            .next()
            .and_then(|last| last.strip_suffix("us"))
            .and_then(|us| us.parse().ok())
            .unwrap_or_else(|| panic!("no microseconds at the end of {said}"));
        assert!(
            us >= TAKES.as_micros() as u64,
            "the write took at least {:?} and the line says {us}us",
            TAKES,
        );
        // And the run is still being played, which is the other half of taking the write off the frame:
        // the frames went by while the disk was busy.
        let from = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            from + 30,
            "the game waited for a disk that is not in its frame",
        );
    });
}

/// How long the disk in that e2e test takes over the file: longer than a frame, and short enough that a
/// suite waiting for it twice over is not something anybody notices.
const TAKES: std::time::Duration = std::time::Duration::from_millis(40);

#[test]
fn the_chapter_written_down_is_written_from_a_thread_of_orbs_own() {
    in_its_own_process(|| {
        // `verbose`, because the line saying the file was written is one of those: it is written once a
        // chapter, and this is what says which thread wrote it and how long the write took.
        let game = Fake::attach("written-off-the-frame", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();

        // The chapter the midboss's card begins, which is a chapter deep enough to have buttons behind
        // it: the file holds every frame of them.
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
        });
        // Read on the frame the chapter began on and not after waiting for the write: what the file
        // holds is that frame, and one more frame of the run is another.
        let at_the_card = game.state();
        assert_eq!(at_the_card.stage_frames, CARD_STARTS);
        // The write is not the frame's, so what a test waits for is the line the writer says it with
        // rather than the frame that handed it over.
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!(
                "chapter 3 (MIDBOSS SPELL 1) at frame {CARD_STARTS}, {CARD_STARTS} frame(s) of buttons"
            ))
        });

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
        // And what it holds is the chapter, not a file the writer left half written: the run is picked
        // up out of it below.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec!["normal-reimu-a".to_owned()],
            "the run is not among the ones left unfinished",
        );

        // The whole of it read back the way a later launch reads it, which is what says the bytes a
        // thread of orb's wrote are the bytes the frame handed over: the run is started again and picked
        // up at that chapter.
        gives_the_run_up(&game);
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state(),
            at_the_card,
            "the run landed in another frame than the one written down off the frame",
        );
    });
}
