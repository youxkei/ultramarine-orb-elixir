//! **Keys another program sent, over th11: the read that sees them is the read this game already makes.**
//!
//! What `keys_from_another_program_into_th10.rs` asks of that game, asked of this one. `GetKeyboardState`
//! reports a key another program sent, and `--sent-keys` is the flag that takes a DirectInput keyboard away from
//! a game so that its read falls back to it. **th11 reads its keys through DirectInput on the settings it comes
//! with**: its input's set-up, run on the main thread in WinMain before `Direct3DCreate9` (0x4456aa through
//! 0x445702), makes the keyboard device at `[0x4c32a0]` in 0x447ab0 and sets bit `0x400` of its input flags' word
//! `[0x4c3810]`, and the read takes `IDirectInputDevice8::GetDeviceState` wherever that bit is set (0x4576d3,
//! 0x457862) and `GetKeyboardState` (0x4576e8) only where it is clear — see `asm.md`'s *The input*.
//!
//! **th10's race is not this game's**: th10 made its device on a thread of its own, so a launch's first frame
//! could come before it; th11's is made before the window has a device to draw with, so it is there before orb's
//! first frame. What is asked of it is that it is let go of all the same.
//!
//! **Whether that device, taken `DISCL_NONEXCLUSIVE | DISCL_FOREGROUND | DISCL_NOWINKEY` (0x447b7b), sees a sent
//! key is not measured**, which is why the flag lets it go rather than trusting it: the laid-out device answers
//! what the hand holds and nothing sent, the stricter of the two.
//!
//! **The keyboard's word is the game's own and only the pad's half is orb's** (0x457270, called with the word in
//! `ecx` at 0x4579eb), so what this file holds is that a sent key reaches the word that half answers.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of th11's own input word the shot button is, which the keyboard's read folds `Z` into (0x4576ee
/// through 0x457848).
const SHOOT: u32 = 0x1;

/// A launch told to expect keys from another program, with a run placed so that the word a key makes is the
/// run's and no question of orb's over the title holds it back.
fn launched(name: &str, sent_keys: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        move |config| {
            config.sent_keys = sent_keys;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game
}

/// **The device the set-up made is let go of**, and the read sent to `GetKeyboardState`.
#[test]
fn th11s_keyboard_device_is_let_go_of() {
    in_its_own_process(|| {
        let game = launched("th11-sent-keys-device", true);
        let log = game.log();
        game.frames_until("the device let go of", 8, || {
            log.said("input: the game's own keyboard device let go of")
        });
        assert!(
            !game.image().reads_its_keyboard_through_directinput(),
            "the input's flags still send the keyboard's read through the device:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And a launch not told to expect them keeps the device**: the flag is what lets it go, and nothing else.
#[test]
fn th11s_keyboard_device_is_kept_where_no_keys_are_to_be_sent() {
    in_its_own_process(|| {
        let game = launched("th11-sent-keys-not-asked", false);
        for _ in 0..8 {
            game.one_frame();
        }
        assert!(
            game.image().reads_its_keyboard_through_directinput(),
            "the keyboard's read was taken off the device with no keys to be sent:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And a sent key reaches the word the game's own read answers, which is what makes an unwatched run
/// driveable.
///
/// Sent rather than pressed, which is the whole of the claim: `orb_sim::Keyboard::held` is what a device that
/// refuses an injected key answers, and nothing here reads that.
#[test]
fn a_key_another_program_sent_reaches_the_word_th11s_own_read_answers() {
    in_its_own_process(|| {
        let game = launched("th11-sent-keys", true);
        game.one_frame();
        assert_eq!(
            game.word_the_input_read_answered() & SHOOT,
            0,
            "the word has the shoot bit in it before anything was sent",
        );

        game.keyboard().sends(keys::Z, true);
        game.frames_until("the sent key reaching the game's own word", 8, || {
            game.word_the_input_read_answered() & SHOOT != 0
        });

        // And it goes when the sending stops, so what the game reads is the state of the keyboard now rather
        // than a key that was sent once.
        game.keyboard().sends(keys::Z, false);
        game.frames_until("the sent key let go of", 8, || {
            game.word_the_input_read_answered() & SHOOT == 0
        });
    });
}
