//! **th07's window going behind and coming forward again: the keys, and the device they are read through.**
//!
//! **The game keeps running while its window is behind**, which is what `always_draw` is and what a launch has by
//! default: a replay or a stress run goes on while attention is elsewhere, and coming back to it is instant rather
//! than a stale frame. So the keys cannot be dealt with by stopping — they are dropped in the input hook.
//!
//! **And this game needs that more than one with a device of its own.** th07 reads its keys with
//! `GetKeyboardState` — 0x430b76 calls the second entry of the USER32 import table at 0x48d1ac with a 0x100-byte
//! buffer, and every key it wants is a byte of it tested against 0x80 — which answers what somebody typed at
//! whatever window is in front. A launch that read it while behind is a launch playing the game with somebody
//! else's typing.
//!
//! Which is also why there is no device to take again on the way back: what th06 re-acquires is a foreground
//! DirectInput device, and `Th07::acquire_input` answers that this game has none. So the keys are read again on
//! the very frame the window comes forward, and nothing is asked of any device.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Another program's window in front. Any handle that is not the game's: what makes it another window is that the
/// host names it as the one in front and not this one.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How many frames the window is left behind for, which is long enough that a launch reading keys there would have
/// read these.
const BEHIND: u32 = 30;

#[test]
fn keys_are_dropped_while_th07s_window_is_behind_and_read_again_the_frame_it_comes_forward() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-window-behind",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
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

        // Behind, with the game still being updated: the frames go on, so it is the input hook that has to drop
        // the keys and not the loop declining to run one.
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

        // And the frames really did go on, which is what makes dropping the keys the only way to answer this: a
        // launch that stopped would have nothing to drop.
        let frames = game.sim().clock().peek();
        game.frames(BEHIND);
        assert!(
            game.sim().clock().peek() > frames,
            "the game stopped being updated while its window was behind",
        );

        // In front again, and the keys are read on that very frame: there is no device to ask for, so nothing is
        // waited for.
        game.sim().display().set_foreground(WINDOW);
        game.one_frame();
        assert_eq!(
            game.word_the_input_read_answered(),
            held,
            "the keys were not read again on the frame the window came forward",
        );
        // And orb says it has the keyboard back all the same, which is the line that marks the frame the keys came
        // back: there was never a device to lose here, and the same line is what a th06 launched with
        // `--sent-keys` writes for the same reason — see `the_window_going_behind.rs`'s second arm.
        assert!(
            log.said("input: keyboard re-acquired"),
            "orb went on holding the keys back with nothing to get back:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And a launch playing itself says it too**, whose word is not the keyboard's and is handed over whichever
/// window is in front.
///
/// A bot's word goes in ahead of everything above — that is what makes it a bot rather than typing — so none of
/// what the window is doing applies to it. **Which is not the same as it not being worth saying.** Somebody
/// watching a launch play itself in a corner of their screen saw its presses come and go with the window's
/// focus, and the log they went to could not say whether the window had been behind at all: the line is written
/// after the bot's word is returned, so a launch that plays itself never reaches it. An observation about a
/// launch is answerable only where the launch says what it did.
#[test]
fn a_launch_playing_itself_says_when_its_window_goes_behind() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-window-behind-a-bot",
            Work::wandering(WORK_US),
            |config| {
                // A recipe of one step that holds a word for longer than this test runs, so the bot's own word
                // is what the read answers on every frame of it.
                config.autoplay = Some("- press: [shot]\n  frames: 6000\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
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
        // **And the word still goes in**, which is the other half of what somebody watching has to be told: the
        // line says where the window is and not that the bot has stopped.
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
