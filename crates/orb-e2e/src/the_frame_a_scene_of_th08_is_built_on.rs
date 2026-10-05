//! **The frame th08's supervisor switches scenes on leaves the keys' words at nothing — th06 1.02h's answer,
//! and not th06c's.**
//!
//! Every case of the supervisor's switch comes through one tail, 0x445adf, which puts the repeat word
//! `[0x164d538]` to nothing and copies it into last frame's word `[0x164d530]` and the read `[0x164d528]`
//! (0x445ae7 through 0x445af9), before `scene = wanted` at 0x445b0b. So a key held across a scene is read at
//! the next read against a last frame of nothing — **an edge**, which is a press to every screen of the front
//! end and to the pause, each of them taking their edges off that pair. Where the switch starts a stage's
//! build the supervisor's update still runs whole on every frame of it, its read at 0x4455f5 among it — its
//! answer of 1 at 0x4455bc being the startup load's alone (0x445584) — so in the game that next read is the
//! frame after the switch.
//!
//! Which is `the_frame_a_scene_is_built_on.rs`'s third test over th06 and the opposite of
//! `the_frame_a_scene_of_th06c_is_built_on.rs`'s, whose game carries the keys straight across; what this
//! holds the laid-out game to is the one this exe has.
//!
//! **What the frame a stage is first played on is here is another file's**: this game builds its stages on a
//! thread and holds the walk while the build stands — see `a_pointdevice_run_of_th08.rs`, whose pick-ups
//! whose build goes on after the stage's jobs are up are the frames that turned on.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run walks into, orb's own 0-based number.
const THE_NEXT_STAGE: i32 = 1;

/// How many frames the switch into the stage's build is given.
const A_SWITCH: u32 = 60;

/// The bit this game's own read leaves for down, which the arrows give at 0x43daf1.
const DOWN: u16 = 0x20;

/// A key held across the switch into the next stage's build: the frame the switch is on leaves both words at
/// nothing, and the frame after it reads the key against that nothing.
#[test]
fn a_scene_switch_of_th08_puts_the_keys_words_to_nothing() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-switch-clears-the-keys",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the run", 60, || game.state().in_run);

        // Held from before the switch and never let go, so that nothing about what the keyboard is doing
        // changes on the frame the scene is switched: what is under test is what the *game* does to the words.
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        game.frame();
        assert_eq!(
            (
                game.image().the_word_the_keys_read() & DOWN,
                game.image().the_word_the_keys_read_last() & DOWN,
            ),
            (DOWN, DOWN),
            "the game's own read did not see the key being held",
        );

        game.moves_on_to_the_stage(THE_NEXT_STAGE);
        game.frames_until("the switch into the build", A_SWITCH, || {
            game.image().the_loader_is_running()
        });
        assert_eq!(
            (
                game.image().the_word_the_keys_read(),
                game.image().the_word_the_keys_read_last(),
            ),
            (0, 0),
            "the frame the scene was switched on came out with the keys' words standing",
        );

        // **And the next read is the key against that nothing, which is an edge** — the supervisor's read on
        // the frame after the switch, the build's first.
        game.frame();
        assert_eq!(
            (
                game.image().the_word_the_keys_read() & DOWN,
                game.image().the_word_the_keys_read_last() & DOWN,
            ),
            (DOWN, 0),
            "the read after the switch did not see the held key as a press",
        );
        game.keyboard().set(keys::DOWN, false);
    });
}
