//! **The name of the spell card a chapter of 妖々夢 is put back inside, which is a sprite and not memory.**
//!
//! The same fault 紅魔郷 has and the same shape of answer, through this game's own calls: 0x428887 bakes a
//! card's name into a vm's own sprite where the card is declared and keeps no copy of the string, so a plate
//! put back into a chapter whose card was declared *before* a later one keeps the later card's name over the
//! card the run is now fighting. See `the_card_name_across_a_restore.rs` for the run that was read off the
//! real 紅魔郷.
//!
//! What orb bakes again is the copy the game itself made in that card's record at 0x40ffb4 — the one copy of
//! a name that is memory — and this asks that it is the *chapter's* card that is named after a restore and not
//! whichever card was declared last.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The two cards the fight puts up, by the record each has of its own: a midboss's and the boss's, named the
/// way 妖々夢 names cards.
const FIRST: usize = 96;
const FIRST_NAME: &str = "幽夢「猫の見た夢」";
const SECOND: usize = 104;
const SECOND_NAME: &str = "反魂「二重の劫火」";

/// The frames the two are put up on, far enough apart that each is a chapter of its own.
const FIRST_AT: u32 = 400;
const SECOND_AT: u32 = 900;

/// Walks the cursor `down` presses from the item it starts on and presses decide there, then waits for what
/// that press was answered with — the shape `a_chapter_further_back.rs` drives these menus by, and for its
/// reason: a press inside the frames a menu reads nothing over is one nothing moved on.
fn chooses(game: &Fake, down: u32, said: &str) {
    let log = game.log();
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    for _ in 0..down {
        game.press(keys::DOWN);
    }
    game.press_until(keys::Z, said, || log.said_since(from, said));
}

/// And answers a confirmation はい, which is one press up from the いいえ its cursor starts on.
fn answers_yes(game: &Fake, said: &str) {
    let log = game.log();
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, said, || log.said_since(from, said));
}

#[test]
fn a_chapter_put_back_inside_a_card_of_th07_has_that_cards_name_on_the_plate() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-card-name",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.in_a_pointdevice_run();

        // The midboss's card, put up the way the game puts one up: the record named, the card declared, and
        // its name baked onto the plate. Which is a chapter of its own — a card starting is a fight moving on.
        game.frames_until("the frames before the first card", FIRST_AT + 8, || {
            game.state().stage_frames >= FIRST_AT
        });
        game.a_card_starts(FIRST, FIRST_NAME);
        game.frames_until("the first card's chapter", 60, || log.said("chapter 2"));
        assert_eq!(
            game.image().the_plate_says(),
            FIRST_NAME.as_bytes(),
            "the card the midboss put up did not name its own plate",
        );

        // And the boss's card after it, which bakes its name over the first's — the plate holding one name at
        // a time is the whole of why a restore has to bake again.
        game.frames_until("the frames before the second card", SECOND_AT + 8, || {
            game.state().stage_frames >= SECOND_AT
        });
        game.a_card_starts(SECOND, SECOND_NAME);
        game.frames_until("the second card's chapter", 60, || log.said("chapter 3"));
        assert_eq!(
            game.image().the_plate_says(),
            SECOND_NAME.as_bytes(),
            "the boss's card did not bake its name over the midboss's",
        );

        // The player dies on the boss's card, and the retry menu offers the chapters behind the one that was
        // lost as well as it: 更に前からやり直す is the item that lists them, one down from the item that puts
        // the lost one back.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
        chooses(&game, 1, "retry: asking which chapter further back");

        // The midboss's card chosen out of them — the newest of the chapters behind, so the item the cursor
        // starts on — and the question that asks before it acts answered はい. Which is the only way back to a
        // card whose name the plate no longer holds.
        chooses(&game, 0, "retry: asking about a chapter further back");
        answers_yes(&game, "retry chapter 2");

        // And the plate says that card's name again, baked out of its own record rather than left saying the
        // card the run was fighting a moment ago.
        assert_eq!(
            game.image().the_plate_says(),
            FIRST_NAME.as_bytes(),
            "the plate kept the later card's name over the card the run is now fighting",
        );
    });
}
