//! **The frame a scene is built on is a frame that scene is updated on, with the input word zeroed.**
//!
//! `Supervisor::OnUpdate` is the first job of the calc chain — `TH_CHAIN_PRIO_CALC_SUPERVISOR` is 0 — and
//! everything it registers goes in at a higher priority, so `Chain::AddToCalcChain` links the new jobs
//! *behind* the walk's current position and `Chain::RunCalcChain` reaches them before it returns. A scene
//! therefore gets its own first update on the very frame it was built. `src/Supervisor.cpp`,
//! `src/Chain.cpp`, `src/ChainPriorities.hpp`.
//!
//! **And the last thing that switch does is zero the word**: `g_CurFrameInput = g_LastFrameInput =
//! g_IsEigthFrameOfHeldInput = 0`, at the end of the state switch and only where a transition happened. So
//! the scene just built is updated with nothing held, and a button still down on the frame after reads as a
//! fresh press rather than as one already spent — which is what keeps a press that chose a menu item from
//! being spent again on whatever the item led to.
//!
//! **Read out of the game's own code rather than off this tree**, because the fake game is both the writer
//! and the reader of the memory orb is driven through: a build frame that skipped the scene's update would
//! be one every scenario here agreed with. The route from an address to the function it is the start of is
//! *Re-deriving it* in
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
//!
//! **It is every transition and not only a stage's.** The front end's build is here beside the stage's for
//! that reason: the two are the same frame of the same function, and a fix that moved one would be expected
//! to move the other.

mod fake;

use fake::th06::{Fake, PLAYER_STARTS_ABOVE, the_run};
use fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::Scene;
use orb_sim::keys;

/// How long each of this game's stages runs here, in its own frames.
const STAGE_FRAMES: u32 = 200;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    })
}

/// A stage's own first update runs on the frame the stage was built on, at a run's start and at a
/// transition alike.
///
/// So `read_state`'s `stage_frames` is already **1** on the frame orb first sees the gameplay scene, not 0:
/// `GameManager::RegisterChain` puts the manager's job in at priority 4 and the stage's at 6, both behind a
/// supervisor sitting at 0, and the walk goes on to them in the same frame.
#[test]
fn a_stages_first_update_runs_on_the_frame_the_stage_was_built_on() {
    in_its_own_process(|| {
        let game = launched("a-built-scene-a-stage");
        game.stages_last(STAGE_FRAMES);
        // The run's first stage, which `in_a_legacy_run` leaves on the frame the stage was built:
        // `frames_until` stops on the frame its condition came true on.
        game.in_a_legacy_run();
        assert_eq!(
            game.state().stage_frames,
            1,
            "the stage the run started did not get its own first update on the frame it was built on",
        );

        // And the transition to the stage after, which is the same callback down its other branch.
        game.frames_until("the stage after the first", STAGE_FRAMES + 60, || {
            game.state().stage == 1
        });
        let after = game.state();
        assert_eq!(
            (after.stage, after.stage_frames),
            (1, 1),
            "the stage a transition built did not get its own first update on the frame it was built on",
        );
    });
}

/// And the front end's own does too, which is the same claim at the transition every launch begins with.
///
/// `MainMenu::RegisterChain` puts its job in at priority 2, behind the same supervisor, so the menu's first
/// update is on the frame the menu was built — its own frame counter is 1 rather than 0 by the time
/// anything above the game can read it.
#[test]
fn the_front_ends_first_update_runs_on_the_frame_it_was_built_on() {
    in_its_own_process(|| {
        let game = launched("a-built-scene-the-front-end");
        // The first frame of the launch. The game is laid out with its front end asked for and not built,
        // which is the supervisor's own first frame — see `Fake::attach`.
        assert!(
            game.image().front_end_now().frames == 0 && game.image().scene() == Scene::FrontEnd,
            "the launch did not begin with the front end asked for and not updated",
        );
        game.frame();
        assert_eq!(
            game.image().front_end_now().frames,
            1,
            "the front end did not get its own first update on the frame it was built on",
        );
    });
}

/// The input word is zeroed on the frame a scene is built, so the scene's first update acts on nothing held.
///
/// A direction held across a stage transition is the way to read it: on the build frame the word is zero and
/// the player is exactly where `Player::AddedCallback` put them, and on the frame after the word holds the
/// key again — which is a fresh press, `g_LastFrameInput` having been zeroed with it.
///
/// The player's position is the second assertion rather than the first because it is the consequence: a
/// fall-through that forgot the zeroing would run the new stage's first update on a button the player never
/// pressed in it, and there is nothing else in the memory that would say so.
#[test]
fn the_input_word_is_zeroed_on_the_frame_a_scene_is_built() {
    in_its_own_process(|| {
        let game = launched("a-built-scene-the-input-word");
        game.stages_last(STAGE_FRAMES);
        game.in_a_legacy_run();

        // Held from before the transition and never let go, so that nothing about what the keyboard is
        // doing changes on the frame the scene is built: it is the *game* that zeroes the word.
        game.frames_until(
            "the last frame of the first stage",
            STAGE_FRAMES + 60,
            || game.state().stage_frames == STAGE_FRAMES - 1,
        );
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        assert_ne!(
            game.image().input_now(),
            0,
            "the game's own read did not see the key being held",
        );
        assert_eq!(
            game.image().scene(),
            Scene::Rebuilding,
            "the first stage did not ask for the next one on the frame this expected",
        );

        // The build frame.
        game.frame();
        assert_eq!(
            game.state().stage,
            1,
            "the transition did not happen on the frame after the one it was asked on",
        );
        assert_eq!(
            game.image().input_now(),
            0,
            "the frame the scene was built on left the input word holding what was pressed",
        );
        let (across, down) = game.image().arcade_region_size();
        assert_eq!(
            unsafe { Th06.reproduction() }.player,
            (across / 2.0, down - PLAYER_STARTS_ABOVE),
            "the new stage's first update moved the player on a key held in the stage before it",
        );

        // And the frame after, where the same key still down is a press the game has not seen before.
        game.frame();
        assert_ne!(
            game.image().input_now(),
            0,
            "the frame after the build did not read the key that is still held",
        );
        game.keyboard().set(keys::DOWN, false);
    });
}
