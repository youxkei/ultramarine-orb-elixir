//! **The frame th10's supervisor switches scenes on leaves the keys' words standing — th06c's answer, and not
//! th08's.**
//!
//! th08's switch puts its keys' words to nothing on its way through every case (0x445adf), so a key held across
//! a scene is a press on the other side of it. **th10's does not**: the switch at 0x4218d0 writes no word of the
//! input at `0x474e30`, whose words the read itself writes (0x44a8bd through 0x44a926, its repeat counted at
//! 0x44a8d2 through 0x44a90a) and whose repeat word the run's copy alone puts at nought besides (0x40ac29) — so a
//! key held across a scene is read against a frame before that held it too, and is a hold
//! on the other side and not a press. Which is `the_frame_a_scene_of_th06c_is_built_on.rs`'s answer, held here to
//! the stage after one won, whose build runs on the loader thread while the supervisor goes on reading.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run walks into, orb's own 0-based number.
const THE_NEXT_STAGE: i32 = 1;

/// How many frames the switch and the build after it are given.
const A_TRANSITION: u32 = 600;

/// The bit this game's own read leaves for down.
const DOWN: u16 = 0x20;

/// A key held across the switch into the next stage's build: no frame of it comes out with the words at
/// nothing, and the first read of the new stage sees the key as held rather than pressed.
#[test]
fn a_scene_switch_of_th10_leaves_the_keys_words_standing() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-switch-keeps-the-keys",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first frames", 120, || {
            game.state().in_run && game.state().stage_frames > 0
        });

        // Held from before the switch and never let go, so that nothing about what the keyboard is doing changes
        // on the frame the scene is switched: what is under test is what the *game* does to the words.
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        game.frame();
        let (now, before) = game.image().the_words_the_keys_read();
        assert_eq!(
            (now & DOWN, before & DOWN),
            (DOWN, DOWN),
            "the game's own read did not see the key being held",
        );

        game.moves_on_to_the_next_stage();
        for _ in 0..A_TRANSITION {
            game.frame();
            let (now, before) = game.image().the_words_the_keys_read();
            assert_eq!(
                (
                    now & DOWN,
                    before & DOWN,
                    game.image().the_buttons_gone_down() & DOWN
                ),
                (DOWN, DOWN, 0),
                "a frame of the switch came out with the held key's words put to nothing, or as a press",
            );
            let state = game.state();
            if state.stage == THE_NEXT_STAGE && state.stage_frames > 0 {
                game.keyboard().set(keys::DOWN, false);
                return;
            }
        }
        panic!("the run never walked into the stage after the first");
    });
}
