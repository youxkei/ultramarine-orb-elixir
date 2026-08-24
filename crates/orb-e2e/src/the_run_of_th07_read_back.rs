//! What th07's memory says a run is, read the way the frame hook reads it.
//!
//! `Th07::read_state` — every offset, every pointer chase, every derived flag — over a game that got where it
//! is by being played, rather than over an image a test wrote fields into. Which is the half the unit tests
//! beside those reads cannot cover: each of them writes a field and reads the `State` back, so orb reading a
//! *different* field from the one the game is in would be one both sides agreed on.
//!
//! What it still cannot catch is an offset that is wrong: the laid-out game lays its memory out through the
//! same constants the reads use, so a wrong one is wrong on both sides at once. Those are settled against
//! `th07.exe` — see `orb_core::game::th07`, where each is written down beside what identifies it, and
//! `docs/adr/0008` for why a laid-out game cannot settle them.
//!
//! The chases are the part worth having here. A field the game has not built yet is reached through a pointer
//! that is nothing, and every one of those has to come back as "none" rather than as a fault: the manager's
//! boss slot before a fight, the dialogue's line through the object the game puts over a run, the laser array,
//! the run's own numbers, and the ending's script — which for this game is found by walking the chain, so
//! "nothing" there is a walk that ends rather than a null.

use crate::fake::th07::{BOMBS, DIFFICULTY, FIRST_STAGE, Fake, LIVES, POWER};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How far into the stage the numbers are read back: far enough that the stage is running, and before this
/// file drives any fight, so what is read is a stage and nothing else.
const AT: u32 = 300;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// The log goes beside the game, which is the one thing about it a game decides: `orb.yaml` and the launcher
/// are there, so that is where somebody looks for what a run did.
#[test]
fn the_log_of_a_launch_of_th07_is_written_beside_the_game() {
    in_its_own_process(|| {
        let game = launched("th07-read-back-log");
        assert_eq!(
            game.log().path(),
            Some(game.dir().join("orb.log")),
            "the log is not beside the exe orb was injected into",
        );
        // The line every run opens with. A log without it cannot say whether the run even happened.
        assert!(game.log().said("---- new run ----"));
        // Appended to rather than started over: a run worth looking at is often over by the time anybody
        // looks.
        assert!(!game.log().restarted());
    });
}

/// Before a run is started, nothing in the game reads as one — and the chases that have nothing to chase come
/// back as nothing rather than faulting.
///
/// Which is what a front end looks like from orb's side: the scene is not the one a stage runs in, so none of
/// the flags a run has is set, and the pointers a run's structures are reached through are the ones the game
/// has not built.
#[test]
fn nothing_is_a_run_of_th07_before_one_is_started() {
    in_its_own_process(|| {
        let game = launched("th07-read-back-title");
        game.image().the_front_end_is_up();
        game.one_frame();
        let state = game.state();
        assert!(!state.playing, "the title menu reads as a stage running");
        assert!(!state.in_run);
        assert!(!state.in_game);
        assert!(!state.replay);
        assert!(!state.demo);
        assert!(!state.practice);
        assert!(!state.paused);
        assert!(!state.in_ending);
        assert_eq!(state.ending_script, None);
        // The manager's boss slot, the object the game puts over a run, the laser array and the boss's own
        // fields: five chases with nothing at the end of them.
        assert!(!state.boss_present);
        assert_eq!(state.boss_life, None);
        assert_eq!(state.boss_attack_frames, None);
        assert_eq!(state.spellcard, None);
        assert!(!state.in_dialogue);
        assert_eq!(state.laser_count, 0);
        // And the run's own numbers, which this game keeps behind a pointer its build allocates: every one of
        // them reads as nothing rather than as a number out of an unbuilt object.
        assert_eq!((state.lives, state.bombs, state.power), (0, 0, 0));
        assert_eq!(state.deaths, 0);
    });
}

/// Every number a stage is played with reads back as the game put it there, at a frame the game reached by
/// being played to it.
///
/// The run's own — the difficulty and the stage it was started for — and the stage's: its two clocks, the
/// numbers a run of this game starts with, and what is on the screen. The stage counts from one while one is
/// running and orb counts from zero, which is the one place the two disagree on purpose.
#[test]
fn every_number_a_stage_of_th07_is_played_with_reads_back_as_the_game_put_it() {
    in_its_own_process(|| {
        let game = launched("th07-read-back-stage");
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames_until("the frame to read", AT + 60, || {
            game.state().stage_frames == AT
        });

        let state = game.state();
        assert!(state.playing && state.in_run && state.in_game);
        assert!(!state.replay && !state.demo && !state.practice);
        assert_eq!(state.stage, FIRST_STAGE, "counted from zero above `Game`");
        assert_eq!(state.difficulty, DIFFICULTY);
        assert_eq!(state.stage_frames, AT);
        assert_eq!(
            state.script_frames, AT as i32,
            "the clock the enemy script runs on is not the stage's own",
        );
        assert_eq!(state.lives, LIVES as i8, "the lives a run starts with");
        assert_eq!(state.bombs, BOMBS as i8);
        assert_eq!(state.power, POWER as u16);
        assert_eq!(state.deaths, 0);
        assert!(
            !state.unsettled,
            "the player is neither dying nor coming back"
        );
        assert!(!state.bombing);
        assert!(!state.in_dialogue, "a stage with no dialogue being played");
        // And no fight, which is what the frame was chosen for.
        assert!(!state.boss_present);
        assert_eq!(state.spellcard, None);

        // The whole of it read again is the same answer: a `State` is read at one instant, so two reads of a
        // game that has not been updated between them cannot differ.
        assert_eq!(game.state(), state, "reading it twice gave two answers");
    });
}

/// A replay being watched is a run — it has stages and chapters — and it is not one orb acts on: no chapter is
/// kept over it and a death offers no menu, there being nobody there to answer one.
///
/// Which is the distinction `in_game` exists for, and the only thing that says it is the game's own flag: a
/// replay looks exactly like play from every other field.
#[test]
fn a_replay_of_th07_is_a_run_but_not_one_orb_acts_on() {
    in_its_own_process(|| {
        let game = launched("th07-read-back-replay");
        let log = game.log();
        game.watches_a_replay();
        game.frames(AT);

        let state = game.state();
        assert!(state.replay, "the run is not a replay being watched");
        assert!(
            state.in_run,
            "a replay is a run: it has stages, and chapters could be taken over it",
        );
        assert!(
            !state.in_game,
            "a replay is not a run somebody is playing, which is what decides whether orb acts on it",
        );
        assert!(
            !log.said("chapter 1 (stage start)"),
            "a chapter was kept over a replay nobody asked to be tracked:\n  {}",
            log.lines().join("\n  ")
        );

        // And the player being hit is the recording being watched, not a chapter to go back to.
        game.dies();
        game.one_frame();
        assert_eq!(game.state().deaths, 1, "the death is the replay's own");
        assert!(
            !log.said("died in chapter"),
            "a replay was offered the retry menu:\n  {}",
            log.lines().join("\n  ")
        );
        let died_at = game.state().stage_frames;
        game.frames(10);
        assert!(
            game.state().stage_frames > died_at,
            "the replay stopped, which is what a retry menu would do to it",
        );
    });
}
