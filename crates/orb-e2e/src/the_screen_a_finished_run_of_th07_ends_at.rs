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
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The number the front end is on when the game is back at its title, which is th07's own 0.
const SCREEN_TITLE: i32 = 0;

/// How many taps a walk off that screen may make before it says the screen took none: each of the screens it
/// is holds its own presses off for its own frames, and a tap is two of them.
const TAPS: u32 = 300;

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

/// **And the screens it is are walked by the presses whoever finished the run makes**, which is how the game
/// gets back to its title: the entry takes a decide on to the offer and the offer takes one to the way out,
/// after which the game puts the scene back itself.
///
/// **A run of the other mode, whose offer nothing of orb's writes past**: a run with chapters has that write
/// take the screen the rest of the way down, so a laid-out game that moved nothing itself still reached the
/// title on one — which is what this e2e test was about before it was about a legacy run.
///
/// Measured on the case that plays a whole game: the real th07 read `front end: screen 0, cursor on item 0`
/// after the run ended and the recipe's own tap was what got it there, where a laid-out game that took no
/// press at that screen sat on it until the case's whole bound ran out — the step aiming at the title gave up
/// and every step after it had nothing to press at.
#[test]
fn and_a_decide_walks_it_off_to_the_title() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-screen-a-run-ends-at-walked",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_legacy_run();
        game.the_run_ends_at_its_own_screen();
        // From the offer rather than from the entry, which is where the roll of a cleared run hands this
        // screen over: what the case that plays a whole game taps at is a screen already past its entry.
        game.the_screen_offers_a_replay();
        // Tapped the way the recipe taps, which is what a screen of the game's own takes: its presses are
        // edges, so a word held across the screens between the entry and the offer takes one of them.
        for _ in 0..TAPS {
            if game.image().the_scene_now() == game.image().scene_of_the_front_end() {
                break;
            }
            game.press(keys::Z);
        }
        assert_eq!(
            game.image().the_scene_now(),
            game.image().scene_of_the_front_end(),
            "the screen a run ended at was still up after {TAPS} tap(s):\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_TITLE,
            "the front end came back somewhere other than its title",
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

/// And a finished legacy run's, whose offer nothing writes past: the screen walks all of itself out, and the
/// file it writes on the way down is the game's own.
///
/// The mode makes two differences and this is where both are visible. Nothing of orb's is in the way of that
/// screen at all — no chapters, so no offer to take off and no question of orb's over it — and the file the
/// screen's own write lands in is `score.dat` rather than the mode's own beside it.
#[test]
fn a_finished_legacy_run_of_th07_walks_its_own_screen_out_and_that_screen_writes() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-screen-a-finished-legacy-run",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_legacy_run();
        game.frames(STANDS_FOR);
        assert!(
            !log.said("chapter 1"),
            "a legacy run took a chapter:\n  {}",
            log.lines().join("\n  ")
        );

        // The run over, and the screen its own end arrives on, in the phase a run somebody played arrives on.
        let from = log.written();
        game.the_run_ends_at_its_own_screen();
        let arrived_in = game.image().phase_of_a_screen_from_a_run();
        for _ in 0..STANDS_FOR {
            game.one_frame();
            assert_eq!(
                game.image().the_phase_the_screen_is_in(),
                arrived_in,
                "the screen a legacy run ended at was moved on while its name entry was up",
            );
        }

        // The offer itself, and nothing of orb's written over it: a run in this mode has a replay worth
        // saving, so the offer stands for the game's own walk to answer.
        game.the_screen_offers_a_replay();
        let offering = game.image().the_phase_the_screen_is_in();
        assert_ne!(
            offering,
            game.image().phase_of_a_screen_leaving(),
            "the screen's own walk did not reach the offer",
        );
        for _ in 0..STANDS_FOR {
            game.one_frame();
            assert_eq!(
                game.image().the_phase_the_screen_is_in(),
                offering,
                "the offer to save a replay was written past in a run with no chapters",
            );
        }
        assert!(
            !log.said_since(from, "replay: the offer to save one was taken off"),
            "orb took the game's own offer off a legacy run's screen:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said_since(from, "asking whether to keep it"),
            "orb asked about a replay of its own for a run it kept none of:\n  {}",
            log.lines().join("\n  ")
        );

        // And what the screen writes on the way down lands in the game's own file: a legacy run is one
        // anybody could have played, so its score belongs in the ranking the game keeps.
        assert_eq!(
            game.opens_its_score_file(true),
            "score.dat",
            "the write of a finished legacy run's screen did not land in the game's own file",
        );
    });
}
