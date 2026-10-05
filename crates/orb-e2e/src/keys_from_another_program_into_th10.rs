//! **Keys another program sent, over th10: the read that sees them is the read this game already makes.**
//!
//! What `keys_from_another_program_into_th08.rs` asks of that game, asked of this one. `GetKeyboardState`
//! reports a key another program sent, and `--sent-keys` is the flag that takes a DirectInput keyboard away from
//! a game so that its read falls back to it. **th10 reads its keys through DirectInput on the settings it comes
//! with**: its input's set-up 0x43ba90, run on a thread of its own (`_beginthreadex` at 0x420249), makes the
//! keyboard device at `[0x491c38]` and sets bit `0x200` of its input flags' word `[0x491ff4]`, and the read takes
//! `IDirectInputDevice8::GetDeviceState` wherever that bit is set (0x44a628) and `GetKeyboardState` (0x44a636)
//! only where it is clear — see `asm.md`'s *The input*.
//!
//! **Whether that device, taken `DISCL_NONEXCLUSIVE | DISCL_FOREGROUND` (0x43b8d0), sees a sent key is not
//! measured**, which is why the flag lets it go rather than trusting it: the laid-out device answers what the
//! hand holds and nothing sent, the stricter of the two.
//!
//! **The keyboard's word is the game's own and only the pad's half is orb's** (0x44a190, called with the word at
//! 0x44a8b3), so what this file holds is that a sent key reaches the word that half answers.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of th10's own input word the shot button is, which the keyboard's read folds `Z` into (0x44a63c
/// through 0x44a8b1).
const SHOOT: u16 = 0x1;

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

/// **The device is let go of however late the set-up's thread makes it**: that thread races the game's own
/// first frames, so a launch whose first frame came before the device found nothing to let go of and then had
/// the bit set under it — every sent key after that read through a device that may not see one.
#[test]
fn th10s_keyboard_device_is_let_go_of_though_its_thread_makes_it_after_orbs_first_frame() {
    in_its_own_process(|| {
        let game = launched("th10-sent-keys-device", true);
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

/// And a sent key reaches the word the game's own read answers, which is what makes an unwatched run
/// driveable.
///
/// Sent rather than pressed, which is the whole of the claim: `orb_sim::Keyboard::held` is what a device that
/// refuses an injected key answers, and nothing here reads that.
#[test]
fn a_key_another_program_sent_reaches_the_word_th10s_own_read_answers() {
    in_its_own_process(|| {
        let game = launched("th10-sent-keys", true);
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
