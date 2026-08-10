//! **`--self-check`: every chapter's snapshot restored into the state it was just taken from.**
//!
//! Which is the one moment a restore can be held against what it should have produced without disturbing
//! anything — the memory is already what the snapshot holds, so putting it back has to leave every byte
//! where it was. A region named here is a region the snapshot saved and cannot give back, and that is a
//! chapter which would come back wrong the first time somebody died on it.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap.**
//! `snapshot::fingerprint_untracked` walks every private range the snapshot does *not* cover, and what a
//! simulated Windows answers that walk with is nothing — see `orb_sim::Sim::private_regions`, whose comment
//! says why: the ranges it could report are the test binary's own pages, and every allocation the harness
//! made would read as the game having changed memory behind orb's back. So the half a laid-out game can
//! speak to is `unrestored`, and the counts beside it are what say the other half found nothing rather than
//! not having been asked.

use crate::fake::th06::{CARD, CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Screen;
use orb_sim::keys;

/// What a check that found nothing says, which is the whole line: the two counts of regions and the count
/// of changes in the process heap.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

#[test]
fn every_chapters_snapshot_gives_back_the_state_it_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach("self-check", the_run(), |config| {
            config.self_check = true;
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        // A song under the stage, because a chapter's snapshot is memory *and* sound: the streaming
        // bookkeeping is in the memory the check compares, and a stage with no track leaves that part of
        // every snapshot reading as zeroes on both sides.
        game.streams_its_song(0);

        game.at_the_title_menu();
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode chosen", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the shot type select", 90, || {
            let front = game.image().front_end_now();
            front.screen == Screen::ShotType && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames_until("the stage built", 8, || game.state().playing);

        // The stage's own start and the two chapters of the fight before the card, each of them a snapshot
        // taken and put straight back.
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });
        let at_the_card = game.state();

        // A check for each of the three snapshots the run has taken — the stage's own start, the boss
        // arriving, and the card — and every one of them found nothing: a region that would not restore is
        // named on the line under the counts, so a run of these lines all reading zero is every region of
        // every snapshot so far giving back what it was taken from.
        for chapter in [
            "stage 1 chapter 1 (stage start)",
            "chapter 2 at frame 400 (script 400): a midboss nonspell",
            "chapter 3 at frame 500 (script 500): a midboss spellcard",
        ] {
            assert!(
                log.said(chapter),
                "the run did not take a snapshot at {chapter:?}:\n  {}",
                log.lines().join("\n  ")
            );
        }
        let checks = log
            .lines()
            .iter()
            .filter(|line| line.contains("self_check: "))
            .count();
        assert!(
            checks >= 3,
            "three snapshots were taken and only {checks} of them were checked",
        );
        let wrong: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("self_check: ") && !line.contains(NOTHING_WRONG))
            .collect();
        assert!(
            wrong.is_empty(),
            "a snapshot did not give back the state it was taken from:\n  {}",
            wrong.join("\n  ")
        );

        // And the restore a run really asks for, which is the one the check above is evidence about: 被弾,
        // チャプターをやり直す, and the run back where the chapter began field for field.
        game.hit();
        game.frame();
        assert!(log.said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the chapter did not come back where it began",
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the attempt the retry is was not counted against the card",
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says
        // the run of zeroes above is not a snapshot that never held anything.
        game.frames_until("a chapter past the retry", 400, || {
            log.lines()
                .iter()
                .filter(|line| line.contains("self_check: "))
                .count()
                > checks
        });
        assert!(
            !log.lines()
                .iter()
                .any(|line| line.contains("self_check: ") && !line.contains(NOTHING_WRONG)),
            "a snapshot taken after a restore did not give back its own state:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
