//! **The screen a run of th07 ends at is the game's to walk, and orb writes one state into it.**
//!
//! That screen is several screens one after another — the high-score name entry its register starts it in, the
//! stats, and then the question about saving a replay — and only the last of those is orb's business: a run with
//! chapters has nothing to put in a replay, so the state is written past that question and no part of it is
//! drawn. Every screen before it belongs to whoever just finished the run.
//!
//! What th07 brings of its own is that the screen is *one* object reached two ways, and which of its screens is
//! up is a phase inside it — `Th07`'s `RESULT_FROM_A_RUN` and `RESULT_OFFERING_A_REPLAY`, both read off the exe.
//! So the same field says both "a run somebody played has just finished" and "the offer is up", and a launch
//! that wrote into the wrong one of those would take the name entry away from whoever was standing at it.
//!
//! A run that finished needs no ranking built for it either: the screen it finished at is what writes the score
//! file, by its own deleted callback — `th07.rs`'s own score-file test is where that half is.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::Th07;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the name entry is held up for: long enough that a launch writing into it would have done so, and
/// short enough that a suite waiting for it twice over is not something anybody notices.
///
/// The stretch itself rather than a count of the game's own, because the real name entry stands until somebody
/// has typed a name: what is asserted is that the screen is still there and still on it the whole way.
const STANDS_FOR: u32 = 120;

fn a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game
}

#[test]
fn the_name_entry_a_finished_run_of_th07_arrives_on_stands_and_the_replay_offer_does_not() {
    in_its_own_process(|| {
        let game = a_run("th07-the-screen-a-run-ends-at");
        let log = game.log();
        let from = log.written();
        // The run over, and the screen its own end arrives on.
        game.the_run_ends_at_its_own_screen();

        // The screen as the run's own end left it, frame after frame: orb writes nothing into it while it is the
        // name entry, which is a screen somebody is standing at.
        let arrived_in = game.image().phase_of_a_screen_from_a_run();
        assert_eq!(
            game.image().the_phase_the_screen_is_in(),
            arrived_in,
            "the screen did not come up in the phase a run somebody played arrives on",
        );
        for _ in 0..STANDS_FOR {
            game.one_frame();
            assert_eq!(
                game.image().the_phase_the_screen_is_in(),
                arrived_in,
                "the screen a run ended at was moved on while its name entry was up",
            );
        }
        assert!(
            !log.said_since(from, "replay: the offer to save one was taken off"),
            "the offer was taken off a screen that was not offering anything:\n  {}",
            log.lines().join("\n  ")
        );

        // And the question after it, which is the one state orb writes: the screen is told to leave, said out
        // loud, and the game puts the scene back itself.
        game.the_screen_offers_a_replay();
        game.frames_until("the offer taken off", 8, || {
            log.said_since(
                from,
                "replay: the offer to save one was taken off the screen a run ended on",
            )
        });
        assert_eq!(
            game.image().the_phase_the_screen_is_in(),
            game.image().phase_of_a_screen_leaving(),
            "the offer was left standing",
        );
        // Nothing of it drawn, which is what taking it off inside the same frame is for: the frame the offer was
        // up on is the frame it was written past.
        assert!(
            game.says("Save Replay").is_empty() && game.says("リプレイ").is_empty(),
            "a frame of the offer reached the screen",
        );
    });
}

/// And the run's own end is what takes the file it was written down in away, which is the other half of a run
/// *finishing*: a chapter to come back to is what a run given up leaves, and there is none here.
#[test]
fn a_finished_run_of_th07_leaves_no_chapter_to_pick_up() {
    in_its_own_process(|| {
        let game = a_run("th07-the-screen-a-run-ends-at-file");
        let log = game.log();
        // The chapter the stage's own start is, written down: what a run that *finished* has to leave behind is
        // nothing, and a file that was never there says nothing about that.
        game.frames_until_a_thread("the chapter written down", || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        assert!(
            !orb_core::resume::left(game.dir()).is_empty(),
            "nothing of the run was written down, so nothing says the file went",
        );

        // And the run finishing takes it away: the screen it finished at is the one thing that tells a run over
        // from a run given up, which leaves its chapter where it is.
        game.the_run_ends_at_its_own_screen();
        game.frames_until_a_thread("the file taken away", || {
            orb_core::resume::left(game.dir()).is_empty()
        });
    });
}

/// The name whoever finished the run left in the game's own entry, and the one the settings file was carrying
/// before it: two names, because a file that already agreed with the entry could not say which of the two the
/// replay took.
const TYPED: &str = "REIMU";
const CARRIED: &str = "ゆうか";

/// **And the replay that run leaves is named by the game's own name entry**, which is the first of the screens
/// it ends at: whoever finished the run types their name there for the score, and that is the name the game's
/// own replay of the same run would have been saved under.
#[test]
fn a_replay_of_th07_is_named_by_the_games_own_entry() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-name-its-own-entry-holds",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                // Kept without asking, the question being `the_replay_offered.rs`'s subject: what this is about
                // is the name the file is sealed with.
                config.save_replays = true;
                config.player_name = CARRIED.to_owned();
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames_until_a_thread("the chapter written down", || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });

        // The run over and the screen its own end arrives on, with the entry as whoever finished it left one:
        // the characters they chose and spaces to the end of the field.
        game.the_run_ends_at_its_own_screen();
        game.image().the_name_its_entry_holds(TYPED);
        game.frames_until("the name read out of the entry", 8, || {
            log.said(&format!("name entry says {TYPED:?}"))
        });

        // And then the screen's own walk reaching the question about saving a replay, which is the frame the
        // file is sealed on — two screens after the entry, and the whole reason the name is read before it.
        game.the_screen_offers_a_replay();
        game.frames_until_a_thread("the replay kept", || {
            !orb_core::replay::saved(game.dir()).is_empty()
        });

        let kept = orb_core::replay::saved(game.dir());
        let (slot, _) = kept[0].rsplit_once('-').expect("a kept replay's own name");
        assert_eq!(
            orb_core::replay::read_saved(game.dir(), &kept[0], &Th07)
                .expect("the replay kept")
                .name,
            TYPED,
            "the replay of {slot} is not named by the game's own entry:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
