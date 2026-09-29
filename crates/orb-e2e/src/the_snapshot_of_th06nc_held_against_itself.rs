//! **`--self-check` over th06nc: every chapter's snapshot restored into the state it was just taken from.**
//!
//! Which is the one moment a restore can be held against what it should have produced without disturbing
//! anything — the memory is already what the snapshot holds, so putting it back has to leave every byte where
//! it was. A region named here is a region the snapshot saved and cannot give back, and that is a chapter which
//! would come back wrong the first time somebody died on it.
//!
//! What `the_snapshot_of_th06c_held_against_itself.rs` holds that game to, held to this one — whose snapshot
//! covers more than th06c's: the heap regions `a_span_of_th06nc_held_back_by_its_source.rs` names are this
//! game's own, and a region of them the check cannot give back is exactly the kind of thing a bisect of the
//! restore was needed for once. **So there is no song under this run**, as there is none under th06c's: the
//! track is in a block every restore leaves where it finds it.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap** — see the th06c
//! one; what a laid-out game can speak to is `unrestored`.

use crate::fake::th06nc::{Fake, MIDBOSS_BEATEN, THE_CARD_THE_STAGE_CASTS};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::State;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// What a check that found nothing says, which is the whole line.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

#[test]
fn every_chapter_of_th06nc_gives_back_the_state_its_snapshot_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-self-check",
            Work::wandering(WORK_US),
            |config| {
                config.self_check = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(0);

        // The stage's own start and the first two chapters of the fight, each a snapshot taken and put
        // straight back.
        game.frames_until("the card's chapter", MIDBOSS_BEATEN, || {
            log.lines().iter().any(|line| {
                line.contains("chapter 3 at frame") && line.contains("a midboss spellcard")
            })
        });
        let at_the_card = game.state();

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

        // And the restore a run really asks for: the hit, チャプターをやり直す, and the run back where the
        // chapter began field for field — but the deaths, which orb counts itself.
        let attempts = game.image().card_attempts(THE_CARD_THE_STAGE_CASTS);
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
            game.image().card_attempts(THE_CARD_THE_STAGE_CASTS),
            attempts + 1,
            "the attempt the retry is was not counted against the card",
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says the
        // run of zeroes above is not a snapshot that never held anything.
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
