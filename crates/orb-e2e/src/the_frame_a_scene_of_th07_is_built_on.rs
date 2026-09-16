//! **The frame a scene of th07 is built on is a frame that scene is updated on, with the input words zeroed.**
//!
//! Its supervisor's own job is the first of the calc chain and everything it registers goes in behind the walk's
//! current position, so the walk reaches the new jobs before it returns: a scene gets its own first update on the
//! very frame it was built. Which for this game is the whole of a transition — 0x4380b4 asks for the rebuild
//! scene, 0x42f45d tears the stage down, 0x42f3c5 registers the manager for the next one and 0x4380df puts the
//! scene asked for back — so no frame of one is a frame outside the scene a run is played in.
//!
//! **And the last thing that block does is zero the input words**: 0x438294 clears 0x4b9e5c, copies it into
//! 0x4b9e54 and from there into 0x4b9e4c, and every transition reaches it — 0x4381cc jumps straight there. So the
//! scene just built is updated with nothing held, and a button still down on the frame after reads as a fresh
//! press rather than as one already spent — which is what keeps a press that chose a menu item from being spent
//! again on whatever the item led to.
//!
//! **Read out of the game's own code rather than off this tree**, because the laid-out game is both the writer and
//! the reader of the memory orb is driven through: a build frame that skipped the scene's update, or one that left
//! a word standing, would be one every e2e test here agreed with.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run moves on to, counted from zero the way everything above `Game` counts.
const NEXT: i32 = 1;

/// How far into the first stage the run is played before it moves on.
const INTO_THE_STAGE: u32 = 120;

fn a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames_until("the stage played into", 600, || {
        game.state().stage_frames >= INTO_THE_STAGE
    });
    game
}

/// The stage the transition builds is a stage being played on that very frame: its own clock at nothing, the scene
/// the one a run is played in, and orb reading a run rather than a run that has ended.
#[test]
fn a_stage_of_th07_is_played_on_the_frame_it_was_built_on() {
    in_its_own_process(|| {
        let game = a_run("th07-the-frame-a-stage-is-built-on");
        game.moves_on_to_the_stage(NEXT);

        // Read before another frame is run, which is what "on the frame it was built" means: the state now is the
        // state the build left, and one more frame of the run is another frame's.
        let built = game.state();
        assert_eq!(
            built.stage, NEXT,
            "the stage the build left is not the next"
        );
        assert_eq!(
            built.stage_frames, 0,
            "the stage's own clock is not at the first of its frames",
        );
        assert!(
            built.playing && built.in_run && built.in_game,
            "the frame the stage was built on is not one a run is being played on: {built}",
        );
        // And the frame after it is the stage's second, which is what says the first was not spent on the build.
        game.one_frame();
        assert_eq!(
            game.state().stage_frames,
            1,
            "the stage's own clock did not move on the frame after the build",
        );
    });
}

/// And the input words are nothing on that frame, however hard a key is held.
#[test]
fn the_input_words_of_th07_are_zeroed_on_the_frame_a_scene_is_built() {
    in_its_own_process(|| {
        let game = a_run("th07-the-frame-the-words-are-zeroed");

        // A key held from before the build and never let up: what the game reads on the frame after the build has
        // to be a fresh press rather than one already spent, and what makes it one is the words being cleared.
        game.keyboard().set(keys::Z, true);
        game.one_frame();
        assert_ne!(
            game.image().the_word_the_read_left(),
            0,
            "the key the keyboard is holding is not in the word the read left",
        );

        game.moves_on_to_the_stage(NEXT);
        assert_eq!(
            game.image().the_word_the_read_left(),
            0,
            "the word the read left was not cleared on the frame the scene was built",
        );

        // And the run's own word with it, which is the one a stage acts on: a stage begun with a button still in
        // it is a stage playing a press nobody made in it.
        assert_eq!(
            game.state().stage_frames,
            0,
            "this is not the frame the stage was built on",
        );
    });
}
