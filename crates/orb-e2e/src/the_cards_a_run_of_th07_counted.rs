//! **What a run of th07 counted against its spell cards, in the game's own score file once that run is over.**
//!
//! Every count a run makes is in memory and nowhere else until a save writes it: the game empties all 141 of its
//! `CATK` records as it builds a run (0x42e657, inside `GAME_MANAGER_ADDED`), the run's attempts and captures go
//! into them, and the file is what carries them to the next launch. So a run given up at the title with its
//! counts still only in memory is a run whose cards were never counted at all — which is what a launch of it
//! showed, and what the laid-out game here is written to be able to say.
//!
//! **The save is the game's own, and the file's own records come back over memory inside it.** `0x44a302(2)`
//! reads the file, lays every record it holds over the array in memory, and then writes that array out — see
//! `Th07`'s `SAVE_SCORE_FILE` and `PARSE_CARD_RECORDS`. So the whole question is what memory holds between those
//! two halves, and the laid-out save below walks them in that order for exactly that reason: a save that only
//! wrote memory out could not tell a fix that puts the session's counts at that seam from one that does nothing.
//!
//! The ranking screen is not in this at all, and that is the finding behind the file: neither phase that screen
//! is built with writes anything, so orb driving it to its end wrote nothing whatever it did.

use crate::fake::th07::{CHARACTER, Fake, SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A card of the game's own, and the name it carries — the pair `th07.rs` drives a card with.
const A_CARD: usize = 96;
const A_CARDS_NAME: &str = "幽夢「猫の見た夢」";

/// Whose counts they are: the shot this run is under, which is the one the game counts into.
const THE_RUNS_SHOT: usize = (CHARACTER * 2 + SHOT) as usize;

/// What the mode's own file already holds for that card, which is the history this run's counts are added to: a
/// number each, and different from each other so that an attempt counted as a capture shows up.
const ATTEMPTS_BEFORE: u16 = 7;
const CAPTURES_BEFORE: u16 = 3;

/// The two files a session orb can rewind has, as the game asks for one and orb answers with the other.
const THE_GAMES_OWN: &str = "score.dat";
const THE_MODES_OWN: &str = "pointdevice_score.dat";

/// And another card, which **only the game's own file holds** — with counts far from this run's, so a number of
/// its own turning up in the mode's file is unmistakable.
///
/// The whole of what the second file is here for: `pointdevice_score.dat` is not started as a copy of the game's
/// own, so what it says has been attempted was attempted in a session that could be rewound. The one read that
/// stays the game's own is the front end's unlocks — and that read lays the game's own card records over the ones
/// in memory, which is how a legacy history reaches a save that writes the mode's file. Watched on a launch as
/// `where the file had 125 and 111` for a card no rewound run had ever reached.
const A_CARD_ONLY_THE_GAME_HAS: usize = 40;
const ITS_ATTEMPTS: u16 = 125;
const ITS_CAPTURES: u16 = 111;

#[test]
fn a_run_of_th07_given_up_has_its_cards_counted_into_the_score_file() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-cards-counted",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        // The two files as a launch finds them: the mode's own with this card's own history, and the game's own
        // with a card of its own that no rewound run has ever reached.
        game.image().its_score_file_holds(
            THE_MODES_OWN,
            A_CARD,
            THE_RUNS_SHOT,
            ATTEMPTS_BEFORE,
            CAPTURES_BEFORE,
        );
        game.image().its_score_file_holds(
            THE_GAMES_OWN,
            A_CARD_ONLY_THE_GAME_HAS,
            THE_RUNS_SHOT,
            ITS_ATTEMPTS,
            ITS_CAPTURES,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // And the run counting one of each against it, the way the game counts them: the attempt as the card
        // goes up, and the capture as it is beaten.
        game.the_boss_arrives();
        game.a_card_starts(A_CARD, A_CARDS_NAME);
        game.frames(30);
        game.image().a_card_has_been_captured(A_CARD, THE_RUNS_SHOT);
        game.the_card_is_over();
        // Which stand in the records as **the file's own counts plus this run's**, and not as this run's alone:
        // the build empties every record and reads the file straight back into them, so what a run adds to is the
        // history. Written down here because it is what says the save has to *put these back* rather than add
        // them to the file's again.
        assert_eq!(
            (
                game.image().card_attempts(A_CARD, THE_RUNS_SHOT),
                game.image().card_captures(A_CARD, THE_RUNS_SHOT),
            ),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "the records do not hold the file's own counts with this run's on them",
        );

        // What the two files' opens landed in, said out loud: a laid-out game that read the mode's own file for
        // its unlocks would be one this test's second file says nothing about.
        assert!(
            log.said("score: score.dat opened as the game's own, read for the front end's unlocks"),
            "the front end's own read did not land in the game's own file:\n  {}",
            log.lines().join("\n  ")
        );

        // やめる, which takes the run back to the title.
        let saves = game.image().times_it_was_asked_to_save();
        game.gives_the_run_up();

        // **And the screen a run ends on is not built on the way**, which is a frame or two of its background
        // somebody sees: the scene th07's own pause menu asks for tears the run down and then registers that
        // screen (0x43803c), and the scene the front end is asks for tears it down and registers nothing
        // (0x43801f). A run given up through orb's own menu is not a run anybody is being shown the score of.
        assert_eq!(
            game.result_screens(),
            0,
            "the screen a run ends on was built for a run given up:\n  {}",
            log.lines().join("\n  ")
        );

        // **And no save asked for there**, which is the half of this a launch of th07 taught: that call is the
        // score screen's own added callback and its deleted one behind it, and the deleted one frees the anm files
        // in slots 0x2a to 0x2d (0x44a25a). The game makes it where nothing else is up — the way out, and a run's
        // own build — and orb making it with the front end up left that screen's vms on a file that had gone:
        // `crash: code 0xc0000005 at 0x00450db1`, inside the walk of the 41 vms an update makes.
        assert_eq!(
            game.image().times_it_was_asked_to_save(),
            saves,
            "orb asked the game for a save with the front end up, which frees anm files under it:\n  {}",
            log.lines().join("\n  ")
        );
        // So the file still holds what it held: what the run counted is written down and waiting.
        assert_eq!(
            game.image()
                .its_score_file_counts(THE_MODES_OWN, A_CARD, THE_RUNS_SHOT),
            (ATTEMPTS_BEFORE, CAPTURES_BEFORE),
            "the file was written at the run's end, where the game writes nothing:\n  {}",
            log.lines().join("\n  ")
        );

        // And a look at the ranking screen before the next run, which is where somebody goes to see what they have
        // just done: **the screen shows what this run counted**, the records in memory being what it walks into
        // itself (0x44a145) after the parse has laid the file's own down.
        game.its_ranking_screen_is_brought_up();
        assert_eq!(
            (
                game.image().card_attempts(A_CARD, THE_RUNS_SHOT),
                game.image().card_captures(A_CARD, THE_RUNS_SHOT),
            ),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "the ranking screen shows the file's own counts, not what the session has just counted:\n  {}",
            log.lines().join("\n  ")
        );
        // And what is waiting survives that read, no write following it: the file is still the file, and the save
        // that comes later is what writes.
        assert_eq!(
            game.image()
                .its_score_file_counts(THE_MODES_OWN, A_CARD, THE_RUNS_SHOT),
            (ATTEMPTS_BEFORE, CAPTURES_BEFORE),
            "the file was written at a read no write follows:\n  {}",
            log.lines().join("\n  ")
        );

        // And **the game's own next save carries them**, which is every way those records can be lost: a run's
        // build empties them (0x42e657) and saves first (0x42e646), and the way out saves last (0x434277). So a
        // second run being built is where this run's counts reach the file.
        game.builds_a_stage();
        assert!(
            game.image().times_it_was_asked_to_save() > saves,
            "the game's own build made no save of its own, so nothing here says where the counts go:\n  {}",
            log.lines().join("\n  ")
        );

        // And **spent once**, which is what the note surviving every read but one means: the records go back into
        // memory at each of them, so whatever is up shows the session; the note itself is only let go of where a
        // write follows, and a read that writes nothing must not be what lets go of it.
        assert_eq!(
            log.lines()
                .iter()
                .filter(|line| line.contains("and the save after it is what writes them down"))
                .count(),
            1,
            "what the run counted was let go of at a read no write follows:\n  {}",
            log.lines().join("\n  ")
        );

        // **And nothing of the game's own file in it**, which is what the one read that stays the game's own
        // costs: that read lays its card records over the ones in memory, and the save after it writes memory
        // out to the mode's file. So a card only the game's own file holds is a card the mode's file must still
        // hold nothing for.
        assert_eq!(
            game.image().its_score_file_counts(
                THE_MODES_OWN,
                A_CARD_ONLY_THE_GAME_HAS,
                THE_RUNS_SHOT
            ),
            (0, 0),
            "the game's own file's history reached the mode's own file:\n  {}",
            log.lines().join("\n  ")
        );

        // And the file holding what it held plus what the run counted — the run's one attempt and one capture on
        // top of the history, rather than either of them alone.
        assert_eq!(
            game.image()
                .its_score_file_counts(THE_MODES_OWN, A_CARD, THE_RUNS_SHOT),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "the file does not hold this run's card counts on top of its own:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And the same for the way out the game's own pause menu takes, where the front end reads the file before
/// orb has seen the run end.**
///
/// The other e2e test's give-up is orb's own menu, and the run is still standing when orb notes what it counted.
/// The game's own way out is not: th07 registers its front end inside the transition the run is left through, so
/// the unlocks read that build makes (0x45bff3) runs while the supervisor's scene still answers a run's — and
/// that read lays the file's own card records over the ones in memory. A note taken after it is a note of the
/// *file's* numbers, and the run's counting is in no file and no memory.
///
/// Which two launches of `--autoplay` measured: a run that attempted two of stage 1's cards and was left through
/// the pause menu wrote `score: the records this run left put back over 0 the file's own differ in`, and the
/// launch before it copied the game's own file's history into the mode's file instead.
///
/// So the note is taken at that read rather than after the run — the last moment the run's records are in
/// memory — and here the read is made with the run still in progress, which is what makes this the other test's
/// counterpart rather than a copy of it.
#[test]
fn a_run_of_th07_left_through_the_games_own_menu_has_its_cards_counted_too() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-cards-counted-the-games-way-out",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        game.image().its_score_file_holds(
            THE_MODES_OWN,
            A_CARD,
            THE_RUNS_SHOT,
            ATTEMPTS_BEFORE,
            CAPTURES_BEFORE,
        );
        game.image().its_score_file_holds(
            THE_GAMES_OWN,
            A_CARD_ONLY_THE_GAME_HAS,
            THE_RUNS_SHOT,
            ITS_ATTEMPTS,
            ITS_CAPTURES,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        game.the_boss_arrives();
        game.a_card_starts(A_CARD, A_CARDS_NAME);
        game.frames(30);
        game.image().a_card_has_been_captured(A_CARD, THE_RUNS_SHOT);
        game.the_card_is_over();
        // The premise, stated: the run's own attempt and capture are in the records, on top of the history the
        // mode's own file had.
        assert_eq!(
            (
                game.image().card_attempts(A_CARD, THE_RUNS_SHOT),
                game.image().card_captures(A_CARD, THE_RUNS_SHOT),
            ),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "the run counted nothing into the records, so this test is not about anything:\n  {}",
            log.lines().join("\n  ")
        );

        // **The front end's read, made while the run is still in progress**, which is where the game makes it —
        // and out of the game's own file, that one read staying its whatever the mode.
        //
        // The premise this test turns on, asserted rather than left to the order these lines are written in:
        // orb has not yet seen the run end, so nothing it does at a run's end can be what saves the counting.
        // Move the two lines below this read above it and every assertion after them passes with none of the
        // path this is about being taken.
        assert!(
            !log.said("score: a run ended"),
            "orb had already seen the run end, so this read is not the one this test is about:\n  {}",
            log.lines().join("\n  ")
        );
        let saves = game.image().times_it_was_asked_to_save();
        assert_eq!(
            game.reads_what_its_menus_offer(),
            THE_GAMES_OWN,
            "the front end's own read did not land in the game's own file, so this test is not about \
             anything:\n  {}",
            log.lines().join("\n  ")
        );
        // And the note taken at that read said so, which is the line a launch is read by.
        assert!(
            log.said(
                "score: what a run has counted taken before this read lays the file's own over it"
            ),
            "the note was not taken at the read, so what puts the counting back is something else:\n  {}",
            log.lines().join("\n  ")
        );
        // And what the run counted is **still in memory after it**, which is the whole of the fix: the parse laid
        // the file's own records over the array and orb put the run's back over them, at the one read that is the
        // last moment they are anywhere.
        assert_eq!(
            (
                game.image().card_attempts(A_CARD, THE_RUNS_SHOT),
                game.image().card_captures(A_CARD, THE_RUNS_SHOT),
            ),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "what the run counted was gone from memory at the front end's own read:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            (
                game.image()
                    .card_attempts(A_CARD_ONLY_THE_GAME_HAS, THE_RUNS_SHOT),
                game.image()
                    .card_captures(A_CARD_ONLY_THE_GAME_HAS, THE_RUNS_SHOT),
            ),
            (0, 0),
            "the game's own file's history was left standing in memory for the save to write:\n  {}",
            log.lines().join("\n  ")
        );

        // And then the run gone, the way the game's own quit leaves it: the front end's scene, with no menu of
        // orb's in it and no result screen either.
        let front_end = game.image().scene_of_the_front_end();
        game.image().is_in_scene(front_end, front_end);
        game.frames_until("the run seen to be over", 60, || {
            log.said("score: a run ended")
        });

        // The game's own next save, which is where those counts reach the file.
        game.builds_a_stage();
        assert!(
            game.image().times_it_was_asked_to_save() > saves,
            "the game's own build made no save, so nothing here says where the counts go:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image()
                .its_score_file_counts(THE_MODES_OWN, A_CARD, THE_RUNS_SHOT),
            (ATTEMPTS_BEFORE + 1, CAPTURES_BEFORE + 1),
            "what the run counted was lost to the read the front end made before orb saw the run end:\n  {}",
            log.lines().join("\n  ")
        );
        // And the same guard the other way out has: the read that laid the game's own history over memory must
        // not be what the save writes out.
        assert_eq!(
            game.image().its_score_file_counts(
                THE_MODES_OWN,
                A_CARD_ONLY_THE_GAME_HAS,
                THE_RUNS_SHOT
            ),
            (0, 0),
            "the game's own file's history reached the mode's own file:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And a stage the run walks into writes nothing**: the save, the records emptied and the parse are 0x42e634,
/// called at 0x42eae7 inside the block 0x42e894 jumps over where the scene asked for is the rebuild one. So what
/// a run counted in its first stage is still only in memory once it is in its second, and the file holds what it
/// held.
#[test]
fn a_stage_of_th07_a_run_walks_into_leaves_the_score_file_alone() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-cards-across-a-transition",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.image().its_score_file_holds(
            THE_MODES_OWN,
            A_CARD,
            THE_RUNS_SHOT,
            ATTEMPTS_BEFORE,
            CAPTURES_BEFORE,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        game.the_boss_arrives();
        game.a_card_starts(A_CARD, A_CARDS_NAME);
        game.frames(30);
        game.the_card_is_over();
        game.the_boss_is_beaten();

        let saves = game.image().times_it_was_asked_to_save();
        game.moves_on_to_the_stage(1);
        assert_eq!(
            game.image()
                .its_score_file_counts(THE_MODES_OWN, A_CARD, THE_RUNS_SHOT),
            (ATTEMPTS_BEFORE, CAPTURES_BEFORE),
            "the stage the run walked into wrote the score file:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().card_attempts(A_CARD, THE_RUNS_SHOT),
            ATTEMPTS_BEFORE + 1,
            "the stage the run walked into lost what the run had counted:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().times_it_was_asked_to_save(),
            saves,
            "orb asked for a save at a stage the run walked into:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
