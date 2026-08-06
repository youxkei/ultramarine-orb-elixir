//! What the game's memory says a run is, read the way the frame hook reads it.
//!
//! `Th06::read_state` — every offset, every pointer chase, every derived flag — over a game that got
//! where it is by being played, rather than over an image a test wrote fields into. Which is the half
//! that was missing: the parse itself was covered by writing `Image::playing` and reading the `State`
//! back, and what that could not catch is orb reading a different stage from the one the game is in,
//! because both sides were the test's.
//!
//! What it still cannot catch is an offset that is wrong: the game lays its memory out through the same
//! constants the reads use, so a wrong one is wrong on both sides at once. Those are settled against
//! the real game — see `DONE.md`.
//!
//! The chases are the part worth having here. A field the game has not built yet is reached through a
//! pointer that is nothing, and every one of those has to come back as "none" rather than as a fault:
//! the bosses pointer before a fight, the dialogue index through a `GuiImpl` on the heap, the laser
//! array, and the ending's script.

mod fake;

use fake::{BOSS_ARRIVES, Fake, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::RunStart;
use orb_core::game::th06::image::Screen;
use orb_sim::keys;

/// The run these play: Normal, Reimu A, from stage one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    game
}

/// A run started, which for these is every screen between the title menu and a stage: the mode
/// question answered, the press handed back to the game's own menu, and the shot answered for.
fn start_a_run(game: &Fake) {
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode chosen", || {
        game.log().said("mode: answered on the keyboard")
    });
    game.frames_until("the shot type select", 90, || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.frames_until("the stage built", 8, || game.state().playing);
}

/// The log goes beside the game, which is the one thing about it a game decides: `orb.yaml` and the
/// launcher are there, so that is where somebody looks for what a run did.
#[test]
fn the_log_is_written_beside_the_game() {
    in_its_own_process(|| {
        let game = game("read-back-log");
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

/// Before a run is started, nothing in the game reads as one — and the chases that have nothing to
/// chase come back as nothing rather than faulting.
///
/// Which is what a front end looks like from orb's side: the scene is not the one a stage runs in, so
/// none of the flags a run has is set, and the pointers a run's structures are reached through are the
/// ones the game has not built.
#[test]
fn nothing_is_a_run_before_one_is_started() {
    in_its_own_process(|| {
        let game = game("read-back-title");
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
        // The bosses pointer, the `GuiImpl` on the heap, the laser array and the boss's own fields: four
        // chases with nothing at the end of them.
        assert!(!state.boss_present);
        assert_eq!(state.boss_life, None);
        assert_eq!(state.boss_attack_frames, None);
        assert_eq!(state.spellcard, None);
        assert!(!state.in_dialogue);
        assert_eq!(state.laser_count, 0);
    });
}

/// Every number a stage is played with reads back as the game put it there, at a frame the game
/// reached by being played to it.
///
/// The run's own — the difficulty and the stage it was started for — and the stage's: its two clocks,
/// the lives and bombs a run starts with, and what is on the screen. The stage counts from one while
/// one is running and orb counts from zero, which is the one place the two disagree on purpose.
#[test]
fn every_number_a_stage_is_played_with_reads_back_as_the_game_put_it() {
    in_its_own_process(|| {
        let game = game("read-back-stage");
        start_a_run(&game);
        // Far enough in that the stage is running and no fight has arrived, so what is read is a stage and
        // nothing else.
        let at = BOSS_ARRIVES - 100;
        game.frames_until("the frame to read", at + 60, || {
            game.state().stage_frames == at
        });

        let state = game.state();
        assert!(state.playing && state.in_run && state.in_game);
        assert!(!state.replay && !state.demo && !state.practice);
        assert_eq!(
            state.stage,
            the_run().stage,
            "counted from zero above `Game`"
        );
        assert_eq!(state.difficulty, the_run().difficulty);
        assert_eq!(state.stage_frames, at);
        assert_eq!(
            state.script_frames, at as i32,
            "the clock the enemy script runs on is not the stage's own",
        );
        assert_eq!(state.lives, 2, "the lives a run starts with");
        assert_eq!(state.bombs, 3);
        assert_eq!(state.power, 0);
        assert_eq!(state.deaths, 0);
        assert_eq!(
            state.enemy_count,
            fake::waves(at as i32),
            "the wave this stage has on at that frame",
        );
        assert!(
            !state.unsettled,
            "the player is neither dying nor coming back"
        );
        assert!(!state.bombing);
        // And the fight has not arrived, which is what the frame was chosen for.
        assert!(!state.boss_present);
        assert_eq!(state.spellcard, None);

        // The whole of it read again is the same answer: a `State` is read at one instant, so two reads of
        // a game that has not been updated between them cannot differ.
        assert_eq!(game.state(), state, "reading it twice gave two answers");
    });
}

/// A replay being watched is a run — it has stages and chapters — and it is not one orb acts on: no
/// chapter is kept over it and a death offers no menu, there being nobody there to answer one.
///
/// Which is the distinction `in_game` exists for, and the only thing that says it is the game's own
/// flag: a replay looks exactly like play from every other field.
#[test]
fn a_replay_is_a_run_but_not_one_orb_acts_on() {
    in_its_own_process(|| {
        let game = game("read-back-replay");
        game.watches_a_replay();
        game.frames_until("the stage built", 8, || game.state().playing);
        game.frames(400);

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
            !game.log().said("chapter 1 (stage start)"),
            "a chapter was kept over a replay nobody asked to be tracked: {:?}",
            game.log().lines(),
        );

        // And the player being hit is the recording being watched, not a chapter to go back to.
        game.hit();
        game.frame();
        assert_eq!(game.state().deaths, 1, "the death is the replay's own");
        assert!(
            !game.log().said("died in chapter"),
            "a replay was offered the retry menu: {:?}",
            game.log().lines(),
        );
        let died_at = game.state().stage_frames;
        game.frames(10);
        assert!(
            game.state().stage_frames > died_at,
            "the replay stopped, which is what a retry menu would do to it",
        );
    });
}
