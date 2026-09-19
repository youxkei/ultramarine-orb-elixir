//! **What a run of th06nc counted at a spell card, and that the run after it does not take it away.**
//!
//! This game keeps 134 card records in its game manager — `game_manager::CARD_HISTORY`, 134 of `0x180` —
//! and two things of orb's touch them:
//!
//! - **`Th06nc::count_card_attempt`** adds one to the record of the card that is up where a chapter is
//!   taken again, because a retry is an attempt at that card and the game counts neither: it counts where
//!   a card *starts*, and a chapter beginning inside one never starts it. It counts nothing where no card
//!   is up and nothing against a record the game has not named, an unnamed one's bytes being the
//!   generator's — `0x14003c390`'s layout lays each record over `0xc0` words of it.
//! - **`Th06nc::put_back_the_cards_a_run_left`**, which is what this file is mostly for: the build of a
//!   run's *first* stage reads the score file over all 134 records — `0x14006e310(the parsed file,
//!   manager + 0x968)` at `0x14003c5e7`, on that build's run-start branch and nowhere else — so a run
//!   that counted an attempt loses it the moment the next run starts. orb notes what a run counted as it
//!   ends and puts the note back where that read has just happened, which is where
//!   `Hooks::stage_begun` already stands.
//!
//! **A stage a run walks into is not that moment**, and the laid-out game is held to the same thing: the
//! read is on the branch `0x14003c78b` jumps out of, and a transition takes the other one at
//! `0x14003c78d`. So the note is spent at the one build that reads, and a run's second stage keeps what
//! its first counted without orb doing anything.
//!
//! **And what that read brings in is a file's own counts**, which is the third thing here: the laid-out
//! game lays the copy's `CATK` records over the layout as `0x14006e310` does — see
//! `Image::the_files_card_records_are_laid_over` — so a count of a launch before this one is in the
//! record the run counts on top of, and a put-back that wrote its own count in place of it would lose
//! every attempt ever played.
//!
//! **The retry is answered on the keyboard**, as `a_confirmation_declined_of_th06nc`'s is: the frame's
//! word carries orb's presses for this game, so a question of orb's can be answered without a recipe.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::PLAYER_PLAYING;
use orb_core::game::th06nc::score_file;
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage lasts here, and how many frames each wait is given.
const STAGE_FRAMES: u32 = 3_000;
const FRAMES: u32 = 900;

/// **The card the run attempts and the name the game wrote in its record**, which is what makes the
/// record one `count_card_attempt` will touch: a name and the sum beside it have to agree, or the game
/// reads the slot as another card's — see `Th06nc::count_card_attempt`.
///
/// **Thirty bytes of name, where a record's field is the thirty-six between `+0x18` and the attempts at
/// `+0x3c`** — and a terminator has to fit in them too, which a card's full name in this game does not
/// always leave room for: `月符「ムーンライトレイ」` is thirty-six on its own.
const CARD: usize = 17;
const CARDS_NAME: &str = "「ムーンライトレイ」";

/// The attempts a record holds before anything counts one, and after.
const NONE: u16 = 0;
const ONE: u16 = 1;

/// **And the attempts a file of this game's already holds at that card**, which the run's own count goes
/// on top of — see [`a_count_the_file_of_th06nc_holds_is_what_the_run_counts_on_top_of`].
const ALREADY: u16 = 4;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// A run in its first stage with a named card up, played past the frames nothing can reach its player
/// in — see `a_confirmation_declined_of_th06nc`, whose own wait this is.
fn in_a_run_at_a_card(game: &Fake) {
    let log = game.log();
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    // **The card up before the chapter that will be taken again is kept**, which is what makes the retry's
    // own restore leave it standing: a chapter's snapshot is the fight's fields as they were on the frame it
    // began, so a card set after that snapshot is a card the chapter no longer has when it comes back. Which
    // is what a heavier suite found — these three tests passed on their own and failed beside the restores
    // of `stepping_inside_a_stage_of_th06nc`, `score: no spell card is up; no attempt counted` where the
    // attempt should have been.
    game.image().the_card_is_named(CARD, CARDS_NAME);
    game.image()
        .the_spellcard_is(u32::try_from(CARD).expect("a card of a hundred and thirty-four"));
    game.frames_until("the run's own first chapter", FRAMES, || {
        log.said("stage 1 chapter 1")
    });
    game.frames_until("the player playing", FRAMES, || {
        game.image().the_players_state() == PLAYER_PLAYING
    });
}

/// The run hit and the chapter taken again, answered on the keyboard: the retry menu's first item is
/// the chapter again, so the cursor is where the question puts it and one decide is the whole answer.
fn the_chapter_taken_again(game: &Fake) {
    let log = game.log();
    game.hit();
    game.frames_until("the death seen", 60, || log.said("died in chapter 1"));
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    game.press_until(keys::Z, "the chapter again", || {
        log.said_since(from, "retry chapter 1")
    });
}

/// **A retry at a card counts one attempt at it**, which is the count the rest of this file is about.
#[test]
fn a_chapter_of_th06nc_taken_again_at_a_card_counts_one_attempt_at_it() {
    in_its_own_process(|| {
        let game = launched("th06nc-cards-counted");
        let log = game.log();
        in_a_run_at_a_card(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            NONE,
            "the card had been attempted before the run attempted it",
        );

        the_chapter_taken_again(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            ONE,
            "the retry counted no attempt at the card that was up:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("retry") || line.contains("score:"))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        assert!(
            log.said("retry: attempt 1 at this spell card"),
            "nothing said the attempt was counted",
        );
    });
}

/// **A count the file already holds reaches the run, and the run counts on top of it** — which is the
/// other half of what the put-back is for: the read the put-back stands after is the one that brings a
/// file's counts in, so a count of orb's that replaced them would lose everything played before this
/// launch.
///
/// The record goes in through the game's own file because that is where the copy is put together from
/// until the mode is answered — the read itself is of the copy and not of a file, `0x14006e310` taking
/// the block `0x14006e140` answers.
#[test]
fn a_count_the_file_of_th06nc_holds_is_what_the_run_counts_on_top_of() {
    in_its_own_process(|| {
        let game = launched("th06nc-cards-read-in");
        let log = game.log();
        // The header and the card's own record, and nothing else: this test never reaches a screen a
        // `CLRD` lights an item of.
        game.its_records_are(&[
            score_file::th6k(),
            score_file::catk(
                u16::try_from(CARD).expect("a card of a hundred and thirty-four"),
                CARDS_NAME,
                ALREADY,
            ),
        ]);

        in_a_run_at_a_card(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            ALREADY,
            "the count the file holds did not reach the run's own records:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("score:"))
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        the_chapter_taken_again(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            ALREADY + ONE,
            "the retry did not count its attempt on top of the file's",
        );
        assert!(
            log.said(&format!(
                "retry: attempt {} at this spell card",
                ALREADY + ONE
            )),
            "the attempt counted was not the file's count and one",
        );
    });
}

/// **And the run after it does not take that attempt away**, which is what the put-back is for: the next
/// run's own build reads the score file over all 134 records, so without it the count goes with the run
/// that made it.
#[test]
fn the_run_after_one_of_th06nc_keeps_the_attempts_it_counted() {
    in_its_own_process(|| {
        let game = launched("th06nc-cards-kept");
        let log = game.log();
        in_a_run_at_a_card(&game);
        the_chapter_taken_again(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            ONE,
            "the retry counted no attempt to keep",
        );

        // **The run given up through the menu's own way out**, which is where orb notes what it counted:
        // a second death raises the question again, タイトルに戻る is two items down — three ways on
        // where the chapter that was lost has none behind it, see `a_confirmation_declined_of_th06nc` —
        // and it asks before it acts, so はい is a second decide with the cursor moved off いいえ.
        game.frames_until("the player playing again", FRAMES, || {
            game.image().the_players_state() == PLAYER_PLAYING
        });
        game.hit();
        game.frames_until("the second death seen", 60, || {
            log.said("retry: the run was lost again")
                || log
                    .lines()
                    .iter()
                    .filter(|line| line.contains("died in chapter"))
                    .count()
                    >= 2
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        let from = log.written();
        game.press_until(keys::Z, "the way out asked about", || {
            log.said_since(from, "retry: asking about the run given up")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the run given up", || {
            log.said("retry: the run is given up")
        });
        game.frames_until("the run over", FRAMES, || !game.state().in_run);

        // And another started, whose own build reads the file over every record — `0x14003c5e7` — and
        // whose put-back is what this asks about.
        game.in_a_run_in_the_stage(0);
        game.frame();
        assert!(
            log.said("score: what the run before counted put back over the 134 this build read in"),
            "nothing put the cards the run before counted back:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("score:"))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            ONE,
            "the run after took away the attempt the run before counted",
        );
        // **And the name with it**, which is what `set_captures_keeping_names` is for: a count put back
        // beside a name the game has not written is a record the game reads as another card's and clears.
        assert!(
            log.said("score: what the run before counted put back"),
            "the put-back said nothing",
        );
    });
}
