//! **What a chapter retried leaves in the record of a run, and whether a playback of that record is the run.**
//!
//! A retry is not a thing the record knows about: the buttons are one entry per stage frame, and an attempt
//! that is rewound plays the same frames again and writes over them — so what the file holds is the frames of
//! the attempt that survived, stitched onto the frames before the chapter it was retried at. Which is the
//! whole reason a retried chapter costs nothing there.
//!
//! **But the surviving attempt was not played from the stage's start.** It was played from a snapshot put back
//! at the chapter's own frame, where a playback of the file reaches that frame by playing the buttons under it
//! instead. The two are the same state only as far as the restore puts everything back: whatever it leaves
//! standing from the attempt that was lost is state the surviving attempt was played with and a playback of it
//! has never seen — and from there the same buttons are a different run.
//!
//! `the_run_a_retried_chapter_of_th07_writes_down.rs` is where that told the runs of th07 apart. This game is
//! the one every field of a snapshot was read for first, so what is asked of it is the same walk: a chapter
//! retried, the chapter after it written down, and the run picked up out of that file landing field for field.

use crate::fake::th06::{ATTACK_CHANGES, CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// The frames played into the card's chapter before the hit, so that what the retry writes over is frames of
/// buttons and not the chapter's own first one.
const INTO_THE_CHAPTER: u32 = 60;

/// The hit and then タイトルに戻る, which leaves the chapter behind for the next launch to offer.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.dies();
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

/// **The chapter a hit lands in kept where the launch asked**, as a run of its own named by the stage, the chapter
/// and the frame of the hit — and the very chapter the run wrote down at its start, byte for byte, which is what
/// a later launch picked up from it plays from.
#[test]
fn a_chapter_hit_in_is_kept_as_a_run_of_its_own_where_the_launch_asks() {
    in_its_own_process(|| {
        const HITS: &str = r"C:\kept\hits";
        let game = Fake::attach("a-chapter-hit-in-kept", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.keep_hits_in = Some(HITS.into());
        });
        let log = game.log();
        game.in_a_pointdevice_run();
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
        });
        let written = game
            .dir()
            .join("pointdevice_resume")
            .join("normal-reimu-a.msgpack");
        game.frames_until_a_thread("the chapter written down", || {
            game.sim().files().get(&written).is_some()
        });
        game.frames(INTO_THE_CHAPTER);

        game.dies();
        game.frame();
        // Named by the frame the count of deaths moved on, which is the frame after the one the hit was dealt on.
        let kept = std::path::Path::new(HITS)
            .join(format!("s1-c3-f{}", CARD_STARTS + INTO_THE_CHAPTER + 1))
            .join("pointdevice_resume")
            .join("normal-reimu-a.msgpack");
        assert!(
            log.said(&format!(
                "resume: the chapter hit in kept as {}",
                kept.display()
            )),
            "the chapter hit in was not kept as {}:\n  {}",
            kept.display(),
            log.lines().join("\n  ")
        );
        game.frames_until_a_thread("the chapter hit in kept", || {
            game.sim().files().get(&kept).is_some()
        });
        assert_eq!(
            game.sim().files().get(&kept),
            game.sim().files().get(&written),
            "what was kept for the hit is not the chapter written down at its start:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A chapter retried, and the run picked up out of the file the chapter after it left.
#[test]
fn a_run_whose_chapter_was_retried_is_put_back_at_the_chapter_after_it() {
    in_its_own_process(|| {
        // `self_check`, so that a restore which left anything of the lost attempt standing says so in the log
        // beside the landing this asserts on.
        let game = Fake::attach("retried-then-picked-up", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.self_check = true;
        });
        let log = game.log();
        game.in_a_pointdevice_run();

        // The chapter that is going to be retried — the card the midboss puts up — with frames of its own
        // behind it, and the state it began with read out of the game's memory: that is what a restore has to
        // put back, field for field.
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
        });
        let at_the_card = game.state();
        assert_eq!(at_the_card.stage_frames, CARD_STARTS);
        game.frames(INTO_THE_CHAPTER);

        // The player lost, and the chapter taken again: the memory is put back, so the run is where the
        // chapter began — and the frames played from here write over the ones just lost.
        game.dies();
        game.frame();
        assert!(
            log.said("died in chapter 3"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the run is not where the chapter began, field for field",
        );

        // And the chapter after it, which is the file this run leaves: its buttons are the frames before the
        // retried chapter and the frames of the attempt that survived, stitched together.
        game.frames_until("the chapter after the retry", ATTACK_CHANGES + 400, || {
            log.said(&format!("chapter 4 at frame {ATTACK_CHANGES}"))
        });
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!(
                "at frame {ATTACK_CHANGES}, {ATTACK_CHANGES} frame(s) of buttons"
            ))
        });

        // Picked up, which is where a stitched record tells: the playback reaches the retried chapter's frame
        // by playing the buttons under it, where the run reached it by having its memory put back.
        gives_the_run_up(&game);
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            ATTACK_CHANGES,
            "the run came back at another frame of the stage:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
