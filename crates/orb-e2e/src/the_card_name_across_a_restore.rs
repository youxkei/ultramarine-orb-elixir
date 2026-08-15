//! **The name of the spell card a chapter is put back inside, which is a sprite and not memory.**
//!
//! The game bakes a card's name into a vm's own sprite where the card is declared —
//! `EnemyManager::RunEclInstruction` hands it to `Gui::SetSpellcardName` at 0x409636, which bakes it
//! through `AnmManager::DrawStringFormat` and keeps no copy of the string. A sprite is a Direct3D
//! texture, so no snapshot holds one: put back into a chapter whose card was declared *before* a later
//! one, the plate would keep the later card's name over the card the run is now fighting.
//!
//! **Which is what going back more than one chapter reaches.** A retry of the chapter that was lost is a
//! card whose own name was the last baked, so nothing was ever wrong there; from the boss's spell card
//! back to the midboss's, the plate said the boss's until the next card started. Read off the real game.
//!
//! So a restore bakes it again, out of the name the game itself copied into that card's record — the one
//! copy of it that is memory, and one orb already carries across a restore for the counts beside it.

use crate::fake::th06::{
    CARD, CARD_NAME, CARD_STARTS, Fake, STAGE_BOSS_CARD, STAGE_BOSS_CARD_NAME,
    STAGE_BOSS_CARD_STARTS, the_run,
};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// A run played out to the card the fight the stage ends with puts up, through the midboss's card on the
/// way: two spell card chapters, the second of which baked its name over the first's.
fn fought_to_the_boss_card(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    // The two songs its data names and the fight fought out, which is what brings the second card at
    // all — see `Fake::fights_its_boss_out`.
    game.plays_its_songs();
    game.fights_its_boss_out();
    game.in_a_pointdevice_run();
    let log = game.log();

    game.frames_until("the midboss's card", CARD_STARTS + 400, || {
        log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
    });
    assert_eq!(
        game.card_name_on_the_plate(),
        CARD_NAME,
        "the card the midboss put up did not name its own plate",
    );
    assert_eq!(
        game.image().card_name(CARD).as_deref(),
        Some(CARD_NAME),
        "the game did not copy that name into the card's own record",
    );

    game.frames_until(
        "the card the stage's own boss puts up",
        STAGE_BOSS_CARD_STARTS + 400,
        || log.said(&format!("at frame {STAGE_BOSS_CARD_STARTS}")),
    );
    assert_eq!(
        game.card_name_on_the_plate(),
        STAGE_BOSS_CARD_NAME,
        "the boss's card did not bake its name over the midboss's",
    );
    game
}

/// Going back from the boss's card to the midboss's puts the midboss's name back on the plate.
#[test]
fn a_chapter_put_back_inside_a_card_has_that_cards_name_on_the_plate() {
    in_its_own_process(|| {
        let game = fought_to_the_boss_card("the-card-name");
        let log = game.log();

        // ── 被弾, and the chapters behind the one that was lost: the midboss's card is one of them, and
        // going back to it is a card whose name is not the one on the plate.
        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapters behind it", || {
            log.said_since(from, "retry: asking which chapter further back")
        });
        // Down to the midboss's card, which is the chapter at the frame that card started.
        let onto_the_card = |game: &Fake| {
            game.frames(READS_KEYS_AFTER);
            for _ in 0..chapters_between() {
                game.press(keys::DOWN);
            }
            game.press_until(keys::Z, "the card's chapter asked about", || {
                log.said_since(from, "retry: asking about a chapter further back")
            });
            game.frames(READS_KEYS_AFTER);
            game.press(keys::UP);
            game.press_until(keys::Z, "the card's chapter put back", || {
                log.said_since(from, "retry: a chapter further back on the keyboard")
            });
        };
        onto_the_card(&game);
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS,
            "the run is not in the chapter the midboss's card began:\n  {}",
            log.lines().join("\n  ")
        );

        // The name on the plate is that card's again, and it came from the record the game itself wrote.
        assert_eq!(
            game.card_name_on_the_plate(),
            CARD_NAME,
            "the plate kept the name of a card the run is no longer fighting",
        );
        // Said out loud, which takes a frame to reach the file: what orb writes at this level is held
        // for a frame's own slack — see `log_held_for_the_slack`. One more frame is inside the chapter
        // either way, the card having been declared on the frame the restore put back.
        game.one_frame_to_drain_the_log();
        assert!(
            log.said_since(
                from,
                &format!("card: the name of card {CARD} is on its plate again")
            ),
            "orb did not say it had baked the name again:\n  {}",
            log.lines().join("\n  ")
        );
        // And the record it came out of still holds it, the restore having carried the records across.
        assert_eq!(
            game.image().card_name(CARD).as_deref(),
            Some(CARD_NAME),
            "the name was baked from something other than the card's own record",
        );
        // Which is not the other card's: the boss's record is where its own name stays.
        assert_eq!(
            game.image().card_name(STAGE_BOSS_CARD).as_deref(),
            Some(STAGE_BOSS_CARD_NAME),
        );
    });
}

/// How many presses down the midboss's card is in the chapters listed behind the one that was lost.
///
/// The chapters between them, which this run reaches in order: the card the boss puts up is the newest,
/// and the one before it is the nonspell its arrival began — so the midboss's card is the third of what
/// is listed. Counted here rather than read off the screen because what this e2e test is about is the
/// plate, and `a_chapter_further_back.rs` is what holds the list itself to its order.
fn chapters_between() -> u32 {
    2
}
