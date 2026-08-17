//! A practice run of 妖々夢: one stage played on its own, which is a run orb rewinds and a run it writes
//! nothing down for.
//!
//! **Both halves are the same fact from two sides.** A practice run is a run — the chapters, the snapshots and
//! the retry menu are a stage's whether the stage was reached by playing to it or chosen from a menu. What it
//! has not got is a name: `Game::run_slot` answers `None`, so there is no file for the chapter it is in, and
//! nothing to offer the next time the same stage is practised. It is one stage started again from the game's
//! own menu in less time than a playback takes.
//!
//! Which for this game rests on one bit — 妖々夢 keeps what kind of run this is in a bitfield of its own, and
//! bit 0 is what its title menu sets for Practice Start (see `Th07`'s `G_RUN_KIND`). Both the run's own name
//! and every flag `read_state` answers come out of that bit, so a run whose kind is not read is a practice run
//! written down as a whole one — a file offering to pick up a stage that was never part of a run.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the fight of this stage puts up, by its own record, named the way 妖々夢 names cards, and the shot
/// this run is played with — whose own word of that record is where an attempt is counted.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";
const SHOT: usize = 0;

/// The frames between the stage's own start and the card: past the shortest a chapter may be.
const APART: u32 = 120;

#[test]
fn a_practice_run_of_th07_is_rewound_and_nothing_is_written_down_for_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-practice",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);

        // ── 1. The run, with the bit its title menu sets for the item a stage is practised from: the mode
        // question is the same one — all three items that start something are runs — and answering it is what
        // decides whether this stage can be rewound.
        game.practises_a_stage();
        let state = game.state();
        assert!(state.practice, "the run reads as a whole run");

        // ── 2. Its chapters are a stage's like any other's: the stage's own start, and the card the fight
        // puts up.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || {
            log.said("chapter 2") && log.said("a midboss spellcard")
        });
        let at_the_card = game.state();

        // ── 3. 被弾, and the chapter put back: what the mode promises is the memory as it was, which for a
        // practice run is the same mechanism and the same snapshot as for a whole one.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 2"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the practice run is not where the chapter began, field for field",
        );
        assert_eq!(
            game.image().card_attempts(CARD, SHOT),
            2,
            "the attempt the retry is was not counted against the card",
        );
        // ── 4. And nothing of it on the disk. A run with no slot is not looked for and not written down:
        // nothing was asked about where to start it — which is what says the file was never even looked for —
        // and the chapters it has reached since have left nothing behind.
        assert!(
            !log.said("resume: nothing was left of"),
            "a practice run was looked up as a run that could have been left:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("resume: stage 1 chapter"),
            "a chapter of a practice run was written down:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a practice run left a chapter behind: {:?}",
            orb_core::resume::left(game.dir()),
        );
    });
}
