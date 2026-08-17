//! **A bomb of 妖々夢 is not a frame orb keeps away from either**, and the chapter that begins under one says
//! so.
//!
//! `chapter::guarded` is the list of frames no chapter may begin on — mid-dialogue and paused — and a bomb is
//! deliberately not on it, for the reason its comment records and `a_chapter_under_a_bomb.rs` holds 紅魔郷 to.
//! What this game brings of its own is the flag: 妖々夢 keeps `bombInUse` in its own player, and orb reads a
//! bomb off that word and off nothing else. So a boundary the fight really has begins its chapter whatever the
//! player has just spent, and the flag turns up in the line that carries the frame's state — which is where
//! somebody looking into a boundary they cannot account for reads a bomb back rather than blaming the frame.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be, so that each is
/// a chapter of its own.
const APART: u32 = 120;

/// How long the bomb is left going off: shorter than the frames between the parts of the fight, so that the
/// attack changes *inside* it — which is the whole of what this e2e test is.
const UNDER_THE_BOMB: u32 = APART / 2;

#[test]
fn a_chapter_of_a_fight_of_th07_begins_while_a_bomb_is_going_off() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-bomb",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The card's chapter, which is the one the bomb is spent inside.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || {
            log.said("chapter 2") && log.said("a midboss spellcard")
        });

        // ボム. One fewer on the count, and the flag orb reads a bomb off up in the game's own memory.
        game.frames(APART - UNDER_THE_BOMB);
        let bombs_before = game.state().bombs;
        game.a_bomb_goes_off();
        game.one_frame();
        let state = game.state();
        assert_eq!(state.bombs, bombs_before - 1, "the bomb was not spent");
        assert!(
            state.bombing,
            "the bomb orb reads is not going off: {state}",
        );

        // And the boundary the fight has while it is still going off begins its chapter all the same.
        game.frames(UNDER_THE_BOMB);
        game.the_card_is_over();
        game.the_attack_changes();
        game.frames_until("the chapter the attack change is", APART, || {
            log.said("chapter 3") && log.said("a midboss nonspell")
        });
        assert!(
            game.state().bombing,
            "the bomb was over by the frame the chapter began, so nothing was begun under one",
        );

        // Said out loud in the line that carries the frame's state, which is where a bomb is read back — see
        // `chapter::guarded`.
        game.frames_until("a state line written under the bomb", APART, || {
            log.lines()
                .iter()
                .any(|line| line.contains(" bombing clears="))
        });

        // And the bomb ending is the run going on: what the flag is worth depends on its going out again, and
        // a chapter is due for no reason of the bomb's on the frame it does.
        game.the_bomb_is_over();
        game.one_frame();
        assert!(!game.state().bombing, "the bomb did not end");
        let after = game.state().stage_frames;
        let ended = log.written();
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            after + 10,
            "the game stopped being played when the bomb ended",
        );
        assert!(
            !log.said_since(ended, "chapter 4 at frame"),
            "a bomb ending began a chapter of its own:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
