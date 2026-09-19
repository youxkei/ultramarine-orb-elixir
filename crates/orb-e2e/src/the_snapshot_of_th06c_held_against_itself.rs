//! **`--self-check` over th06c: every chapter's snapshot restored into the state it was just taken from.**
//!
//! Which is the one moment a restore can be held against what it should have produced without disturbing
//! anything — the memory is already what the snapshot holds, so putting it back has to leave every byte where it
//! was. A region named here is a region the snapshot saved and cannot give back, and that is a chapter which
//! would come back wrong the first time somebody died on it.
//!
//! What this game brings of its own is what a snapshot of it *covers*: its nine megabytes of `.data` and the
//! heap blocks beside it — with the renderer's objects, the sound library's block and the statics its track
//! is played out of held back, which is `Th06c::live_handles` and `Th06c::audio_state`. **So there is no song
//! under this run**, where th06's and
//! th07's have one: those two carry the streaming bookkeeping in the memory the check compares, and this game's
//! track is in a block every restore leaves where it finds it — a song here would put nothing into a snapshot
//! for the check to be about.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap.**
//! `snapshot::fingerprint_untracked` walks every private range the snapshot does *not* cover, and what a
//! simulated Windows answers that walk with is nothing — see `orb_sim::Sim::private_regions`. So the half a
//! laid-out game can speak to is `unrestored`, and the counts beside it are what say the other half found
//! nothing rather than not having been asked.

use crate::fake::th06c::{Fake, MIDBOSS_BEATEN, SCRIPTED_CARD};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::State;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// What a check that found nothing says, which is the whole line: the two counts of regions and the count of
/// changes in the process heap.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

#[test]
fn every_chapter_of_th06c_gives_back_the_state_its_snapshot_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-self-check",
            Work::wandering(WORK_US),
            |config| {
                config.self_check = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(0);

        // The stage's own start and the first two chapters of the fight — the midboss arriving and the card
        // it puts up — each of them a snapshot taken and put straight back.
        game.frames_until("the card's chapter", MIDBOSS_BEATEN, || {
            log.lines().iter().any(|line| {
                line.contains("chapter 3 at frame") && line.contains("a midboss spellcard")
            })
        });
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

        // And the restore a run really asks for, which is the one the check above is evidence about: the hit,
        // チャプターをやり直す, and the run back where the chapter began field for field — **but the deaths**,
        // which this game keeps no count of: orb counts them itself (see `Th06c::read_state`), so a chapter
        // put back leaves the count where the hit that lost it put it.
        game.hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            State {
                deaths: at_the_card.deaths,
                ..game.state()
            },
            at_the_card,
            "the chapter did not come back where it began",
        );
        assert_eq!(
            game.image().card_attempts(SCRIPTED_CARD),
            2,
            "the attempt the retry is was not counted against the card",
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says the
        // run of zeroes above is not a snapshot that never held anything: the next chapter is the attack
        // after the card, which the script moves the fight on to.
        game.frames_until("a chapter past the retry", MIDBOSS_BEATEN, || {
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
