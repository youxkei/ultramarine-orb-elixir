//! **th06c's window going behind and coming forward again: the keys a run reads.**
//!
//! What `th07s_window_going_behind.rs` holds that game to, held to this one. **The game keeps running while
//! its window is behind**, so the keys cannot be dealt with by stopping — they are dropped in the read. And
//! this game reads them the way th07 does: `Th06c::acquire_input` takes its DirectInput device away every
//! frame, so what it plays with is `GetKeyboardState` — the one read orb is in front of, see
//! `runtime::keyboard_state` — and that answers what somebody typed at whatever window is in front. A launch
//! that read it while behind is a launch playing the game with somebody else's typing.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Another program's window in front: any handle that is not the game's.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How many frames the window is left behind for.
const BEHIND: u32 = 30;

#[test]
fn keys_are_dropped_while_th06cs_window_is_behind_and_read_again_once_it_comes_forward() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-window-behind",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(0);
        game.frames(2);

        // In front, the word the run acts on is the key that is down: which is what makes the assertion
        // below say something.
        game.keyboard().set(keys::UP, true);
        game.frames(2);
        let held = game.image().the_word_the_run_acts_on();
        assert_ne!(
            held, 0,
            "the key the keyboard is holding is not in the word"
        );

        // Behind, with the game still being updated: the frames go on, so it is the read that has to drop the
        // keys and not the loop declining to run one.
        game.sim().display().set_foreground(ANOTHER);
        game.frames(BEHIND);
        assert!(
            log.said("input: window behind, keys not read"),
            "orb did not notice the window going behind:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            0,
            "a key held into another program's window reached the game",
        );

        // In front again, and the keys are read again.
        game.sim().display().set_foreground(WINDOW);
        game.frames(2);
        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            held,
            "the keys were not read again once the window came forward",
        );
    });
}
