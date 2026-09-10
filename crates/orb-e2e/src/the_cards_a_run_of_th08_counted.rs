//! **The record th08 keeps per spell card, held across what would replace it — and the attempt orb
//! counts where the game cannot.**
//!
//! The game counts an attempt where a card *starts* (`SpellCard::Start` at 0x4152a0), so a card retried
//! ten times from inside is one attempt to it. `Game::count_card_attempt` is what counts the other nine,
//! and for this game that is two writes and a third: the character's own slot at `+0x124`, the total
//! beside it at `+0x154`, and **the byte at `+0xe`** — which is the whole reason this file exists rather
//! than a unit test over the arithmetic.
//!
//! **A count written without that byte is a count the game throws away.** 0x4159a0 through 0x415a3b sums
//! the record's name as signed bytes and then every dword of all six of its arrays, and where the low
//! byte of that sum differs from `+0xe` it zeroes all six (0x415a74). So the test below works the sum out
//! a second time, from `th08::image`'s own literals, and holds the byte orb wrote against it: two
//! implementations of the game's rule, which is the only way to say the byte is right without the game.
//!
//! **And the 222 records taken and put back**, which is what a chapter of this game would carry: the
//! whole block as bytes, with the names left standing where a playback has named them since.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::game::Game;
use orb_core::game::th08::Th08;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A card of this game's own, by the number its record is at, and the name that record carries.
///
/// The number is one of the 222 and not the first, so a record read at the array's base cannot pass for
/// this one; the name is the game's own Japanese, which is what the record really holds.
const A_CARD: usize = 96;
const A_CARDS_NAME: &str = "「蓬莱の薬」";

/// And a second card, so that a put-back over the whole block is held to more than one record.
const ANOTHER_CARD: usize = 3;
const ANOTHER_CARDS_NAME: &str = "夜符「ムーンライトレイ」";

/// Whose run it is, which is the slot the counts are kept in — the fourth of the twelve, so a count read
/// at slot 0 cannot pass for it.
const A_CHARACTER: u8 = 3;

/// And the thirteenth slot, which is the total across the twelve.
const EVERY_CHARACTER: usize = 12;

/// The attempts and captures the records start with, chosen so no two numbers here are the same.
const ATTEMPTS: u32 = 7;
const ATTEMPTS_OF_EVERY_CHARACTER: u32 = 19;
const CAPTURES: u32 = 2;

fn in_a_run_with_a_card_up(name: &str) -> Box<Fake> {
    let game = Fake::attach(Display::agreed(120), name, Work::wandering(WORK_US));
    game.is_in_a_run();
    let image = game.image();
    image.its_card_records_are_stamped();
    image.the_character_playing_is(A_CHARACTER);
    for (card, named) in [(A_CARD, A_CARDS_NAME), (ANOTHER_CARD, ANOTHER_CARDS_NAME)] {
        image.a_cards_record_is_named(card, named);
        image.a_card_has_been_attempted(card, usize::from(A_CHARACTER), ATTEMPTS);
        image.a_card_has_been_attempted(card, EVERY_CHARACTER, ATTEMPTS_OF_EVERY_CHARACTER);
        image.a_card_has_been_captured(card, usize::from(A_CHARACTER), CAPTURES);
    }
    image.a_card_is_up(A_CARD);
    game.frame();
    game
}

/// **The attempt counted, both slots, and the record left in a state the game would accept.**
#[test]
fn an_attempt_at_a_card_of_th08_is_counted_and_the_record_stamped() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th08-a-card-counted");
        let image = game.image();

        // The card orb reads is the card that is up, which is what `State` carries.
        assert_eq!(
            game.state().spellcard,
            Some(A_CARD as u32),
            "orb does not read the card the game has up",
        );

        let counted = unsafe { Th08.count_card_attempt() };
        assert_eq!(
            counted,
            Some(ATTEMPTS as u16 + 1),
            "the count orb answered is not the character's own, one higher",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_CARD, usize::from(A_CHARACTER)),
            ATTEMPTS + 1,
            "the character's own attempts did not go up",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_CARD, EVERY_CHARACTER),
            ATTEMPTS_OF_EVERY_CHARACTER + 1,
            "the total across the twelve did not go up with it",
        );
        // And the captures left alone, which is what an attempt is not.
        assert_eq!(
            image.the_captures_of_a_card(A_CARD, usize::from(A_CHARACTER)),
            CAPTURES,
            "counting an attempt moved the captures",
        );

        // **And the byte the game checks it against**, worked out a second time from the image's own
        // literals: a record orb left unstamped is one the game zeroes the next time it looks.
        assert_eq!(
            image.the_sum_a_cards_record_carries(A_CARD),
            image.the_sum_the_games_rule_gives_a_cards_record(A_CARD),
            "the record orb counted in does not carry the byte the game's own rule gives it",
        );

        // And the card beside it untouched, orb having counted one card and not the array.
        assert_eq!(
            image.the_attempts_at_a_card(ANOTHER_CARD, usize::from(A_CHARACTER)),
            ATTEMPTS,
            "counting one card's attempt moved another's",
        );
    });
}

/// **And nothing counted where no card is up**, and nothing counted for a practice of one card, which is
/// the gate the game's own count has.
#[test]
fn no_attempt_at_a_card_of_th08_is_counted_where_there_is_none_to_count() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th08-no-card-counted");
        let image = game.image();

        image.no_card_is_up();
        assert_eq!(
            game.state().spellcard,
            None,
            "orb reads a card that is not up"
        );
        assert_eq!(
            unsafe { Th08.count_card_attempt() },
            None,
            "orb counted an attempt at a card that is not up",
        );

        // And a practice of one card, where the game counts its attempts elsewhere: the card is up and
        // the answer is still nothing.
        image.a_card_is_up(A_CARD);
        image.the_run_is_a_spell_practice();
        assert_eq!(
            unsafe { Th08.count_card_attempt() },
            None,
            "orb counted an attempt in a run that is a practice of one card",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_CARD, usize::from(A_CHARACTER)),
            ATTEMPTS,
            "a run that is a practice of one card had its attempts moved",
        );
    });
}

/// **The whole block taken and put back**, which is what a chapter of this game would carry.
#[test]
fn the_card_records_of_th08_are_taken_and_put_back_whole() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th08-the-records-put-back");
        let image = game.image();

        let taken = unsafe { Th08.captures() };
        assert!(
            Th08.these_records_hold_a_count(&taken),
            "records holding an attempt read as holding no count",
        );

        // Every count moved, and both of the cards' — so a put-back that reached one record cannot pass.
        for card in [A_CARD, ANOTHER_CARD] {
            image.a_card_has_been_attempted(card, usize::from(A_CHARACTER), ATTEMPTS + 40);
            image.a_card_has_been_captured(card, usize::from(A_CHARACTER), CAPTURES + 40);
        }

        unsafe { Th08.set_captures(&taken) };
        assert_eq!(
            unsafe { Th08.captures() },
            taken,
            "the records put back do not read back as the records that were taken",
        );
        for card in [A_CARD, ANOTHER_CARD] {
            assert_eq!(
                (
                    image.the_attempts_at_a_card(card, usize::from(A_CHARACTER)),
                    image.the_captures_of_a_card(card, usize::from(A_CHARACTER)),
                ),
                (ATTEMPTS, CAPTURES),
                "card {card}'s counts are not the ones the records were taken with",
            );
        }
    });
}

/// **And put back with the names a playback wrote left standing** — which is the one thing about a card
/// the game itself knows and the session needs.
#[test]
fn the_card_records_of_th08_go_back_with_the_names_they_now_carry() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th08-the-names-kept");
        let image = game.image();

        // Taken before the card was ever named, which is the state a session starts a card in.
        image.a_cards_record_is_named(A_CARD, "");
        let taken = unsafe { Th08.captures() };

        // Then the card started, which is what names the record and counts the attempt.
        image.a_cards_record_is_named(A_CARD, A_CARDS_NAME);
        image.a_card_has_been_attempted(A_CARD, usize::from(A_CHARACTER), ATTEMPTS + 1);

        unsafe { Th08.set_captures_keeping_names(&taken) };
        assert_eq!(
            image.the_name_a_cards_record_carries(A_CARD),
            A_CARDS_NAME,
            "the name the card wrote went back with the counts",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_CARD, usize::from(A_CHARACTER)),
            ATTEMPTS,
            "the count the playback made was not put back",
        );
        // **And the byte stamped for the pair**, which is what the name from now and the counts from then
        // make necessary: neither the byte that was taken nor the byte before it is the right one.
        assert_eq!(
            image.the_sum_a_cards_record_carries(A_CARD),
            image.the_sum_the_games_rule_gives_a_cards_record(A_CARD),
            "the record put back with its name does not carry the byte the game's own rule gives it",
        );
    });
}

/// **And a stamped record holds no count**, which is what says the question is not a scan for bytes: the
/// init at 0x43bbe1 leaves every record full of its own tag, length, version and index.
#[test]
fn the_card_records_of_th08_a_launch_stamped_hold_no_count() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-records-stamped",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.image().its_card_records_are_stamped();
        game.frame();

        let stamped = unsafe { Th08.captures() };
        assert!(
            stamped.iter().any(|byte| *byte != 0),
            "a stamped record array is empty, which is not what the game's own init leaves",
        );
        assert!(
            !Th08.these_records_hold_a_count(&stamped),
            "records a launch had only stamped read as holding a count",
        );
    });
}
