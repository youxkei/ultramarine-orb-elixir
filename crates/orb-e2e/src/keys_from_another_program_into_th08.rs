//! **Keys another program sent, over th08: the read that sees them is the read this game already makes.**
//!
//! What `keys_from_another_program_into_th07.rs` asks of that game, asked of this one — and the answer is
//! th07's. `GetKeyboardState` reports a key another program sent where a DirectInput device held
//! `DISCL_EXCLUSIVE | DISCL_FOREGROUND` reports only the hand, and `--sent-keys` is the flag that takes such a
//! device away from th06. **th08 reads its keys with `GetKeyboardState` already**: 0x43d996 calls the second
//! entry of the USER32 import table at 0x4b41c8 with a 0x100-byte buffer and tests each key's byte against
//! 0x80 — see `Th08::acquire_input`, and `Th08::take_sent_keys` answering that nothing was let go of. So a
//! sent key drives this game with or without the flag.
//!
//! **There is a DirectInput path behind `0x17ce768` all the same**, and orb stands in front of the whole read
//! (0x43d970), so which of the two the game would have taken never reaches it: what this file holds is that a
//! sent key reaches the word that read answers.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of th08's own input word the shot button is: `Z` gives it at 0x43dbd7.
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

/// The flag is answered with what this game is: no device to let go of, and the read that sees a sent key
/// already being the one it makes.
#[test]
fn th08_is_said_to_be_reading_the_other_way_already() {
    in_its_own_process(|| {
        let game = launched("th08-sent-keys-none", true);
        let log = game.log();
        game.frames_until("orb's one ask about the device", 8, || {
            log.said("input: the game had no keyboard device")
        });
        assert!(
            log.said(
                "input: the game had no keyboard device to let go of; it was already reading the \
                 other way"
            ),
            "orb did not say the game had none:\n  {}",
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
fn a_key_another_program_sent_reaches_the_word_th08s_own_read_answers() {
    in_its_own_process(|| {
        let game = launched("th08-sent-keys", true);
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

/// And a launch that was *not* told to expect them reads them all the same, because the read is the same read.
#[test]
fn a_key_another_program_sent_reaches_th08_without_the_flag_as_well() {
    in_its_own_process(|| {
        let game = launched("th08-sent-keys-unasked", false);
        game.one_frame();
        game.keyboard().sends(keys::Z, true);
        game.frames_until("the sent key reaching the game's own word", 8, || {
            game.word_the_input_read_answered() & SHOOT != 0
        });
    });
}
