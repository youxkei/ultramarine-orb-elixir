//! **The game's window going behind and coming forward again: the keys, and the device they are read
//! through.**
//!
//! What each e2e test holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! **The game keeps running while its window is behind**, which is what `always_draw` is and what a
//! launch has by default: a replay or a stress run goes on while attention is elsewhere, and coming back
//! to it is instant rather than a stale frame. So the keys cannot be dealt with by stopping — they are
//! dropped in the input hook — and the device they would have been read through has been taken away by
//! the system in the meantime, because the game holds it `DISCL_EXCLUSIVE | DISCL_FOREGROUND`.
//!
//! Which is the pair of things here: nothing is read while the window is behind, and the device is asked
//! for again before anybody reads it on the way back. The other way round — `always_draw` off, where the
//! frame is paced and the game asked for nothing at all — is `pacing.rs`'s
//! `a_frame_with_the_window_behind_takes_its_turn_on_the_blanks`.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, WINDOW, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_core::game::th06::image::item;
use orb_sim::keys;

/// Another program's window in front. Any handle that is not the game's: what makes it another window is
/// that the host names it as the one in front and not this one.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// How many frames the window is left behind. Long enough that a frame of them would have been noticed:
/// what must not happen is one of them acting on a key.
const BEHIND: u32 = 30;

/// The keys are dropped while the window is behind, and the keyboard device is taken again on the way
/// back — as many times as it takes.
///
/// **Measured, which is why orb asks rather than leaving it to the game**: the game's own read checks
/// only for `DIERR_INPUTLOST`, the one answer that reports a loss, and treats anything else as a
/// success — so a read of an unacquired device hands it an uninitialised stack buffer as the key state.
/// Whether the game ever sees that one report depends on exactly where the frames fell, which is not
/// something to leave a whole keyboard resting on.
#[test]
fn keys_are_dropped_behind_and_the_keyboard_device_is_taken_again_in_front() {
    in_its_own_process(|| {
        let game = Fake::attach("the-window-behind", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.at_the_title_menu();
        assert!(
            game.image().holds_a_keyboard_device(),
            "the game has no device for the system to take away",
        );
        assert_eq!(
            game.keyboard_acquires(),
            0,
            "the device was asked for with the window in front all along",
        );
        let cursor = game.image().front_end_now().cursor;

        // Behind, with the game still being updated: the frames go on, so it is the input hook that has
        // to drop the keys and not the loop declining to run one.
        game.sim().display().set_foreground(ANOTHER);
        game.frames(BEHIND);
        game.press(keys::DOWN);
        assert!(
            game.log().said("input: window behind, keys not read"),
            "orb did not notice the window going behind:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.image().front_end_now().cursor,
            cursor,
            "the title menu acted on a key pressed into another program's window",
        );

        // In front again, with the device still refusing — `DIERR_OTHERAPPHASPRIO`, which is what one
        // whose window has only just come forward answers. The game is handed nothing on that frame, and
        // orb asks again on the next: giving up after one refusal is the game reading an unacquired
        // device for the rest of the run.
        game.refuses_the_keyboard_acquire(true);
        game.sim().display().set_foreground(WINDOW);
        game.frame();
        assert_eq!(
            game.keyboard_acquires(),
            1,
            "the device was not asked for on the frame the window came forward",
        );
        assert!(
            !game.log().said("input: keyboard re-acquired"),
            "orb said it had the device back while that device was refusing:\n  {}",
            game.log().lines().join("\n  ")
        );
        game.frame();
        assert_eq!(
            game.keyboard_acquires(),
            2,
            "orb gave up on a device that refused once",
        );

        // And once it takes, orb says so and stops asking: the acquire is a call into DirectInput on the
        // game's own thread, and one a frame for the rest of the run is a cost nothing is buying.
        game.refuses_the_keyboard_acquire(false);
        game.frame();
        assert!(
            game.log().said("input: keyboard re-acquired"),
            "orb did not say it had the device back:\n  {}",
            game.log().lines().join("\n  ")
        );
        let asked = game.keyboard_acquires();
        game.frames(BEHIND);
        assert_eq!(
            game.keyboard_acquires(),
            asked,
            "orb went on asking for a device it already had back",
        );

        // Which is the whole point of asking: the keys are read again, so the menu moves on a press.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::GAME_START + 1,
            "the menu did not read the keyboard again after the window came forward",
        );
    });
}

/// And a launch that has let the game's own device go has nothing to acquire, which is a success rather
/// than a failure to read anything.
///
/// `--sent-keys` clears the pointer the game's read branches on — see
/// `keys_from_another_program.rs` — and what is left is `GetKeyboardState`, which the system
/// never takes away. So the window coming forward asks nothing of any device and the game is handed the
/// keys on that very frame.
#[test]
fn a_launch_with_no_device_of_the_games_has_nothing_to_take_again() {
    in_its_own_process(|| {
        let game = Fake::attach("the-window-behind-sent-keys", the_run(), |config| {
            config.sent_keys = true;
            config.log_level = LogLevel::Verbose;
        });
        game.frames_until("the device let go of", 8, || {
            !game.image().holds_a_keyboard_device()
        });
        game.at_the_title_menu();
        let cursor = game.image().front_end_now().cursor;

        game.sim().display().set_foreground(ANOTHER);
        game.frames(BEHIND);
        game.sim().display().set_foreground(WINDOW);
        // The device that would have refused is not there to be asked, and orb says it has the keyboard
        // back all the same: there was never anything to lose.
        game.refuses_the_keyboard_acquire(true);
        game.frame();
        assert_eq!(
            game.keyboard_acquires(),
            0,
            "a device the game no longer holds was asked to be acquired",
        );
        assert!(
            game.log().said("input: keyboard re-acquired"),
            "orb went on holding the keys back with no device to get back:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the keys are read, through `GetKeyboardState`: sent rather than pressed, since that is the
        // read that sees a key another program sent and the whole reason the device was let go.
        game.keyboard().sends(keys::DOWN, true);
        game.frame();
        game.keyboard().sends(keys::DOWN, false);
        game.frame();
        assert_eq!(
            game.image().front_end_now().cursor,
            cursor + 1,
            "the menu did not read the keys the other way after the window came forward",
        );
    });
}
