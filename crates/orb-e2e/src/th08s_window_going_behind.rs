//! **th08's window going behind and coming forward again: the keys, and the device they are read through.**
//!
//! What `th07s_window_going_behind.rs` asks of that game, asked of this one. **The game keeps running while its
//! window is behind**, which is what `always_draw` is and what a launch has by default, so the keys cannot be
//! dealt with by stopping — they are dropped in the input hook.
//!
//! **And this game needs that as much as th07 does.** th08 reads its keys with `GetKeyboardState` — 0x43d996
//! calls the second entry of the USER32 import table at 0x4b41c8 with a 0x100-byte buffer — which answers what
//! somebody typed at whatever window is in front. A launch that read it while behind is a launch playing the
//! game with somebody else's typing.
//!
//! Which is also why there is no device to take again on the way back: `Th08::acquire_input` answers that the
//! keyboard needs nothing done to it, so the keys are read again on the very frame the window comes forward.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Another program's window in front: any handle that is not the game's.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How many frames the window is left behind for, which is long enough that a launch reading keys there would
/// have read these.
const BEHIND: u32 = 30;

#[test]
fn keys_are_dropped_while_th08s_window_is_behind_and_read_again_the_frame_it_comes_forward() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-window-behind",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // A run placed, so that the word a key makes is the run's: at the title a held decide is one orb's own
        // question may hold back, which is `the_mode_question_over_th08.rs`'s subject and not this one.
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.one_frame();

        // In front, the word the game's own read answers is the key that is down: which is what makes the
        // assertion below say something.
        game.keyboard().set(keys::Z, true);
        game.one_frame();
        let held = game.word_the_input_read_answered();
        assert_ne!(
            held, 0,
            "the key the keyboard is holding is not in the word"
        );

        // Behind, with the game still being updated: the frames go on, so it is the input hook that has to
        // drop the keys and not the loop declining to run one.
        game.sim().display().set_foreground(ANOTHER);
        game.frames(BEHIND);
        assert!(
            log.said("input: window behind, keys not read"),
            "orb did not notice the window going behind:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.word_the_input_read_answered(),
            0,
            "a key held into another program's window reached the game",
        );

        let frames = game.sim().clock().peek();
        game.frames(BEHIND);
        assert!(
            game.sim().clock().peek() > frames,
            "the game stopped being updated while its window was behind",
        );

        // In front again, and the keys are read on that very frame: there is no device to ask for, so nothing
        // is waited for.
        game.sim().display().set_foreground(WINDOW);
        game.one_frame();
        assert_eq!(
            game.word_the_input_read_answered(),
            held,
            "the keys were not read again on the frame the window came forward",
        );
        assert!(
            log.said("input: keyboard re-acquired"),
            "orb went on holding the keys back with nothing to get back:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And a launch playing itself says it too**, whose word is not the keyboard's and is handed over whichever
/// window is in front — the line being the only way somebody watching can tell whether the window was behind
/// at all.
#[test]
fn a_launch_of_th08_playing_itself_says_when_its_window_goes_behind() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-window-behind-a-bot",
            Work::wandering(WORK_US),
            |config| {
                // A recipe of one step that holds a word for longer than this test runs, so the bot's own
                // word is what the read answers on every frame of it.
                config.autoplay = Some("- press: [shot]\n  frames: 6000\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.one_frame();
        let held = game.word_the_input_read_answered();
        assert_ne!(held, 0, "the bot's own word is not in the read");

        game.sim().display().set_foreground(ANOTHER);
        game.frames(BEHIND);
        assert!(
            log.said("input: window behind, keys not read"),
            "a launch playing itself did not say its window had gone behind:\n  {}",
            log.lines().join("\n  ")
        );
        // **And the word still goes in**: the line says where the window is and not that the bot has stopped.
        assert_eq!(
            game.word_the_input_read_answered(),
            held,
            "the bot's word was dropped with the window behind",
        );

        game.sim().display().set_foreground(WINDOW);
        game.frames(2);
        assert!(
            log.said("input: window in front"),
            "the window coming forward was not said:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
