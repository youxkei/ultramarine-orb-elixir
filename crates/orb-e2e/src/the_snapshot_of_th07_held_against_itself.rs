//! **`--self-check` over th07: every chapter's snapshot restored into the state it was just taken from.**
//!
//! Which is the one moment a restore can be held against what it should have produced without disturbing
//! anything — the memory is already what the snapshot holds, so putting it back has to leave every byte where it
//! was. A region named here is a region the snapshot saved and cannot give back, and that is a chapter which
//! would come back wrong the first time somebody died on it.
//!
//! What this game brings of its own is what a snapshot of it *covers*: its `.data`, the sprite manager's block
//! the frame queues quads in, the objects a stage allocates, and the streaming bookkeeping of a track read out
//! of `thbgm.dat`. So the run below has a song under it — a stage with no track leaves that part of every
//! snapshot reading as zeroes on both sides, which is a check that cannot fail.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap.**
//! `snapshot::fingerprint_untracked` walks every private range the snapshot does *not* cover, and what a
//! simulated Windows answers that walk with is nothing — see `orb_sim::Sim::private_regions`. So the half a
//! laid-out game can speak to is `unrestored`, and the counts beside it are what say the other half found
//! nothing rather than not having been asked.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way th07 names cards, and the shot this run is
/// played with — whose own word of that record is where an attempt is counted.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";
const SHOT: usize = 0;

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// What a check that found nothing says, which is the whole line: the two counts of regions and the count of
/// changes in the process heap.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

#[test]
fn every_chapter_of_th07_gives_back_the_state_its_snapshot_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-self-check",
            Work::wandering(WORK_US),
            |config| {
                config.self_check = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The stage's own start and the two chapters of the fight after it, each of them a snapshot taken and
        // put straight back.
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
        let at_the_card = game.state();

        // A check for each of the three snapshots the run has taken, and every one of them found nothing: a
        // region that would not restore is named on the line under the counts, so a run of these lines all
        // reading zero is every region of every snapshot so far giving back what it was taken from.
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

        // And the restore a run really asks for, which is the one the check above is evidence about: The hit,
        // チャプターをやり直す, and the run back where the chapter began field for field.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the chapter did not come back where it began",
        );
        assert_eq!(
            game.image().card_attempts(CARD, SHOT),
            2,
            "the attempt the retry is was not counted against the card",
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says the
        // run of zeroes above is not a snapshot that never held anything.
        game.frames(APART);
        game.the_card_is_over();
        game.the_attack_changes();
        game.frames_until("a chapter past the retry", APART, || {
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
