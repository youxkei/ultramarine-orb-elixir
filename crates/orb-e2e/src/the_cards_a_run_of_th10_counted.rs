//! **The entries th10 keeps per spell card, held across what would replace them — and the attempt orb
//! counts where the game cannot.**
//!
//! The game counts an attempt where a card *starts* (0x409373 through 0x4093b8), so a card retried ten times
//! from inside is one attempt to it. `Game::count_card_attempt` is what counts the other nine, and for this game
//! that is two writes: the attempts at `+0x84` of the card's entry in the run's shot type's record, and in the
//! seventh record's, which is every shot type's — each held at 99,999 as the game holds them, and neither made
//! in a replay being played (0x409312).
//!
//! **There is no check byte to keep here**, unlike th08's: a record's sum at `+0x4` is checked as the file is
//! parsed (0x42b0a5 through 0x42b109), and the card's start writes its counts with no sum beside them (0x409373
//! through 0x4093b8) — so what the tests below hold is the two counts and the entries beside them left alone.
//!
//! **And the seven records' entries taken and put back**, which is what a chapter of this game carries: every
//! entry as bytes, with the names left standing where a playback has named them since.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::game::Game;
use orb_core::game::th10::Th10;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A card by its number, and a name for its entry: the number one of the 110 and not the first, so an entry read
/// at the array's base cannot pass for this one; the name any bytes, what the test asks of them being that they
/// stand.
const A_CARD: usize = 20;
const A_CARDS_NAME: &[u8] = b"a card's name of this test's own";

/// And a second card, so that a put-back over the whole of them is held to more than one entry.
const ANOTHER_CARD: usize = 3;
const ANOTHER_CARDS_NAME: &[u8] = b"another card's name";

/// Whose run it is — Marisa with her third shot, `1 * 3 + 2` — which is the record the counts are kept in, so a
/// count read at the first record cannot pass for it.
const A_CHARACTER: i32 = 1;
const A_SHOT: i32 = 2;
const A_SHOT_TYPE: usize = 5;

/// And the seventh record, which is every shot type's.
const EVERY_SHOT_TYPE: usize = 6;

/// The attempts and captures the entries start with, chosen so no two numbers here are the same.
const ATTEMPTS: u32 = 7;
const ATTEMPTS_OF_EVERY_SHOT_TYPE: u32 = 19;
const CAPTURES: u32 = 2;

/// Where the game stops counting (`cmp 0x1869f` at 0x409380 and 0x4093b0).
const A_COUNT_STOPS_AT: u32 = 99_999;

fn in_a_run_with_a_card_up(name: &str) -> Box<Fake> {
    let game = Fake::attach(Display::agreed(120), name, Work::wandering(WORK_US));
    game.is_in_a_run();
    let image = game.image();
    image.the_character_is(A_CHARACTER);
    image.the_shot_is(A_SHOT);
    for (card, named) in [(A_CARD, A_CARDS_NAME), (ANOTHER_CARD, ANOTHER_CARDS_NAME)] {
        image.a_cards_entry_is_named(A_SHOT_TYPE, card, named);
        image.a_card_has_been_attempted(A_SHOT_TYPE, card, ATTEMPTS);
        image.a_card_has_been_attempted(EVERY_SHOT_TYPE, card, ATTEMPTS_OF_EVERY_SHOT_TYPE);
        image.a_card_has_been_captured(A_SHOT_TYPE, card, CAPTURES);
    }
    image.a_card_is_up(A_CARD as i32);
    game.frame();
    game
}

/// **The attempt counted, in both records, and nothing else moved.**
#[test]
fn an_attempt_at_a_card_of_th10_is_counted_in_its_shot_types_record_and_every_shot_types() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th10-a-card-counted");
        let image = game.image();

        // The card orb reads is the card that is up, which is what `State` carries.
        assert_eq!(
            game.state().spellcard,
            Some(A_CARD as u32),
            "orb does not read the card the game has up",
        );

        let counted = unsafe { Th10.count_card_attempt() };
        assert_eq!(
            counted,
            Some(ATTEMPTS as u16 + 1),
            "the count orb answered is not the shot type's own, one higher",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_SHOT_TYPE, A_CARD),
            ATTEMPTS + 1,
            "the shot type's own attempts did not go up",
        );
        assert_eq!(
            image.the_attempts_at_a_card(EVERY_SHOT_TYPE, A_CARD),
            ATTEMPTS_OF_EVERY_SHOT_TYPE + 1,
            "every shot type's attempts did not go up with it",
        );
        // And the captures left alone, which is what an attempt is not, and the name with them.
        assert_eq!(
            (
                image.the_captures_of_a_card(A_SHOT_TYPE, A_CARD),
                image.the_name_a_cards_entry_carries(A_SHOT_TYPE, A_CARD),
            ),
            (CAPTURES, A_CARDS_NAME.to_vec()),
            "counting an attempt moved the captures or the name",
        );

        // And the card beside it untouched, orb having counted one card and not the array.
        assert_eq!(
            image.the_attempts_at_a_card(A_SHOT_TYPE, ANOTHER_CARD),
            ATTEMPTS,
            "counting one card's attempt moved another's",
        );
    });
}

/// **And held where the game holds it**, a count at 99,999 left there.
#[test]
fn an_attempt_at_a_card_of_th10_stops_where_the_games_own_count_stops() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th10-a-card-counted-at-its-top");
        let image = game.image();
        image.a_card_has_been_attempted(A_SHOT_TYPE, A_CARD, A_COUNT_STOPS_AT);
        image.a_card_has_been_attempted(EVERY_SHOT_TYPE, A_CARD, A_COUNT_STOPS_AT);
        unsafe { Th10.count_card_attempt() };
        assert_eq!(
            (
                image.the_attempts_at_a_card(A_SHOT_TYPE, A_CARD),
                image.the_attempts_at_a_card(EVERY_SHOT_TYPE, A_CARD),
            ),
            (A_COUNT_STOPS_AT, A_COUNT_STOPS_AT),
            "a count the game holds at 99,999 went past it",
        );
    });
}

/// **And nothing counted where no card is up**, and nothing counted in a replay being played, which is the gate
/// the game's own count has.
#[test]
fn no_attempt_at_a_card_of_th10_is_counted_where_there_is_none_to_count() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th10-no-card-counted");
        let image = game.image();

        image.no_card_is_up();
        game.frame();
        assert_eq!(
            game.state().spellcard,
            None,
            "orb reads a card that is not up"
        );
        assert_eq!(
            unsafe { Th10.count_card_attempt() },
            None,
            "orb counted an attempt at a card that is not up",
        );

        image.a_card_is_up(A_CARD as i32);
        image.the_run_is_a_replay_being_played();
        assert_eq!(
            unsafe { Th10.count_card_attempt() },
            None,
            "orb counted an attempt in a replay being played",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_SHOT_TYPE, A_CARD),
            ATTEMPTS,
            "a replay being played had its attempts moved",
        );
    });
}

/// **Every record's entries taken and put back**, which is what a chapter of this game carries.
#[test]
fn the_card_entries_of_th10_are_taken_and_put_back_whole() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th10-the-entries-put-back");
        let image = game.image();

        let taken = unsafe { Th10.captures() };
        assert!(
            Th10.these_records_hold_a_count(&taken),
            "entries holding an attempt read as holding no count",
        );

        // Every count moved, and both of the cards' — so a put-back that reached one entry cannot pass.
        for card in [A_CARD, ANOTHER_CARD] {
            image.a_card_has_been_attempted(A_SHOT_TYPE, card, ATTEMPTS + 40);
            image.a_card_has_been_captured(A_SHOT_TYPE, card, CAPTURES + 40);
        }

        unsafe { Th10.set_captures(&taken) };
        assert_eq!(
            unsafe { Th10.captures() },
            taken,
            "the entries put back do not read back as the entries that were taken",
        );
        for card in [A_CARD, ANOTHER_CARD] {
            assert_eq!(
                (
                    image.the_attempts_at_a_card(A_SHOT_TYPE, card),
                    image.the_captures_of_a_card(A_SHOT_TYPE, card),
                ),
                (ATTEMPTS, CAPTURES),
                "card {card}'s counts are not the ones the entries were taken with",
            );
        }
    });
}

/// **And put back with the names a playback wrote left standing** — which is the one thing about a card the
/// game itself knows and the session needs.
#[test]
fn the_card_entries_of_th10_go_back_with_the_names_they_now_carry() {
    in_its_own_process(|| {
        let game = in_a_run_with_a_card_up("th10-the-names-kept");
        let image = game.image();

        // Taken before the card was ever named, which is the state a session starts a card in.
        image.a_cards_entry_is_named(A_SHOT_TYPE, A_CARD, b"");
        let taken = unsafe { Th10.captures() };

        // Then the card started, which is what names the entry and counts the attempt.
        image.a_cards_entry_is_named(A_SHOT_TYPE, A_CARD, A_CARDS_NAME);
        image.a_card_has_been_attempted(A_SHOT_TYPE, A_CARD, ATTEMPTS + 1);

        unsafe { Th10.set_captures_keeping_names(&taken) };
        assert_eq!(
            image.the_name_a_cards_entry_carries(A_SHOT_TYPE, A_CARD),
            A_CARDS_NAME,
            "the name the card wrote went back with the counts",
        );
        assert_eq!(
            image.the_attempts_at_a_card(A_SHOT_TYPE, A_CARD),
            ATTEMPTS,
            "the count the playback made was not put back",
        );
    });
}

/// **And records the game made empty hold no count**, which is what says the question is not a scan for bytes:
/// 0x42acb0 leaves every entry with its card's number and difficulty at `+0x88` and `+0x8c`.
#[test]
fn the_card_entries_of_th10_a_launch_made_empty_hold_no_count() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-entries-made-empty",
            Work::wandering(WORK_US),
        );
        game.image()
            .the_score_object_is_read_from(&orb_core::game::th10::score_file::Records::default());
        game.is_in_a_run();
        game.frame();

        let emptied = unsafe { Th10.captures() };
        assert!(
            emptied.iter().any(|byte| *byte != 0),
            "an emptied record's entries are all nought, which is not what the game's own code leaves",
        );
        assert!(
            !Th10.these_records_hold_a_count(&emptied),
            "entries a launch had only made empty read as holding a count",
        );
    });
}
