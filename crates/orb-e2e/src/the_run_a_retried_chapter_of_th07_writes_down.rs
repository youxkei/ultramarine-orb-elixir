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
//! Which is the one thing telling the runs that resume from what a session found: a 妖々夢 run of stage 1 with
//! `0 retries` was picked up and reproduced every one of its 2667 frames, generator for generator, while a run
//! of stage 6 with 13 of them parted inside its first chapter and the player was hit at frame 1249.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The cards the midboss puts up, by its own record, named the way 妖々夢 names cards: one for the chapter that
/// is retried and one for the chapter after it, so the file the run leaves is written past the retry.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";
const LATER_CARD: usize = 97;
const LATER_CARD_NAME: &str = "夢符「刈安色の江戸紫」";

/// The frames between one part of a stage and the next: past the shortest a chapter may be, so each is a
/// chapter of its own with frames of buttons behind it.
const APART: u32 = 120;

/// A chapter retried, and the run picked up out of the file the chapter after it left.
#[test]
fn a_run_of_th07_whose_chapter_was_retried_is_put_back_at_the_chapter_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-retried-then-picked-up",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                config.self_check = true;
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The chapter that is going to be retried, with frames of its own behind it.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at_the_card = game.state();
        game.frames(APART);

        // The player lost, and the chapter taken again: the memory is put back, so the run is where the
        // chapter began — and the frames played from here write over the ones just lost.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 2"));
        game.frames(READS_KEYS_AFTER);
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
        game.frames(APART);
        game.a_card_starts(LATER_CARD, LATER_CARD_NAME);
        game.frames_until("the chapter after the retry", APART, || {
            log.said("chapter 3")
        });
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // Picked up, which is where a stitched record tells: the playback reaches the retried chapter's frame
        // by playing the buttons under it, where the run reached it by having its memory put back.
        game.gives_the_run_up();
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            game.state().stage_frames, at,
            "the run came back at another frame of the stage",
        );
    });
}
