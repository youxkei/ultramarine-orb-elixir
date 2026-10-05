//! **th10's window going behind and coming forward again: the keys, and the device they are read through.**
//!
//! What `th08s_window_going_behind.rs` asks of that game, asked of this one. **The game keeps running while its
//! window is behind**, which is what `always_draw` is and what a launch has by default, so the keys cannot be
//! dealt with by stopping — they are dropped in the input hook, which over this game is the pad's half of the read
//! (0x44a190) that the keyboard's word is handed to.
//!
//! **And this game skips its keyboard while behind itself**, `[0x4924fc]` being the window's `WM_ACTIVATEAPP`
//! (0x44a61d) — which a launch set to `always_draw` cannot lean on, the pad's half answering whatever window is
//! in front.
//!
//! **And there is a device to take again on the way back, which the game takes itself.** On the settings th10
//! comes with it reads its keys through DirectInput (0x44a628), a keyboard held `DISCL_FOREGROUND` (0x43b8d0)
//! that the system takes away while the window is behind: the first read in front again fails, acquires, and
//! answers nothing for that frame (0x44a766, 0x44a77a), and the keys are read on the frame after. orb's hook
//! being the half of the read after the keyboard's, nothing of orb's can come before that — see
//! `Th10::acquire_input`.

use crate::fake::th10::Fake;
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
fn keys_are_dropped_while_th10s_window_is_behind_and_read_again_once_its_device_is_back() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-window-behind",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // A run placed, so that the word a key makes is the run's: at the main menu a held decide is one orb's
        // own question may hold back, which is `the_mode_question_over_th10.rs`'s subject and not this one.
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

        // In front again: the frame the game's own read spends acquiring the device, and the keys read on the
        // one after.
        game.sim().display().set_foreground(WINDOW);
        game.one_frame();
        assert_eq!(
            game.word_the_input_read_answered(),
            0,
            "the keys came through a device the system had taken away while the window was behind",
        );
        game.one_frame();
        assert_eq!(
            game.word_the_input_read_answered(),
            held,
            "the keys were not read again on the frame after the window came forward",
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
fn a_launch_of_th10_playing_itself_says_when_its_window_goes_behind() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-window-behind-a-bot",
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
