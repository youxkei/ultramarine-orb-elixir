//! **Keys another program sent, over 妖々夢: the read that sees them is the read this game already makes.**
//!
//! **A key another program sent is not a key a hand is holding, and the difference is which read sees it.**
//! `GetKeyboardState` reports both; a DirectInput device held `DISCL_EXCLUSIVE | DISCL_FOREGROUND` reports only
//! the hand — which is what `orb_sim::Keyboard`'s two reads are, and the measurement behind them is written there.
//!
//! What `--sent-keys` does to 紅魔郷 is take that device away so the game falls back to `GetKeyboardState`.
//! **妖々夢 is already there**: 0x430b76 calls the second entry of the USER32 import table at 0x48d1ac — which is
//! `GetKeyboardState` — with a 0x100-byte buffer, and every key it wants is a byte of it tested against 0x80. So
//! the flag has nothing to let go of, and a sent key drives this game with or without it.
//!
//! Why it matters past automation: every measurement of a front end that nobody was there to press keys at was
//! taken through a sent key, so a launch that cannot be driven is a launch that cannot be measured.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of 妖々夢's own input word the key below is: what its default mapping calls shoot — see
/// `Th07::pad_word` and `th07.rs`'s own pad test, which reads the same bit off the same word.
const SHOOT: u16 = 0x1;

/// A launch told to expect keys from another program.
fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.sent_keys = true;
            config.log_level = LogLevel::Verbose;
        },
    )
}

/// The flag is answered with what this game is: no device to let go of, and the read that sees a sent key
/// already being the one it makes.
#[test]
fn th07_is_said_to_be_reading_the_other_way_already() {
    in_its_own_process(|| {
        let game = launched("th07-sent-keys-none");
        let log = game.log();
        // Asked on the frame hook rather than at the attach, because the device it is about does not exist until
        // the game has made one — which for this game is never.
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

/// And a sent key reaches the word the game's own read answers, which is what makes an unwatched run driveable.
///
/// Sent rather than pressed, which is the whole of the claim: `orb_sim::Keyboard::held` is what a device that
/// refuses an injected key answers, and nothing here reads that.
#[test]
fn a_key_another_program_sent_reaches_the_word_th07s_own_read_answers() {
    in_its_own_process(|| {
        let game = launched("th07-sent-keys");
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

        // And it goes when the sending stops, so what the game reads is the state of the keyboard now rather than
        // a key that was sent once.
        game.keyboard().sends(keys::Z, false);
        game.frames_until("the sent key let go of", 8, || {
            game.word_the_input_read_answered() & SHOOT == 0
        });
    });
}

/// And a launch that was *not* told to expect them reads them all the same, because the read is the same read.
///
/// Which is what makes the flag cost this game nothing: there is no launch of 妖々夢 in which a sent key is
/// invisible, so `--sent-keys` is neither needed nor harmful here.
#[test]
fn a_key_another_program_sent_reaches_th07_without_the_flag_as_well() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-sent-keys-unasked",
            Work::wandering(WORK_US),
        );
        game.one_frame();
        game.keyboard().sends(keys::Z, true);
        game.frames_until("the sent key reaching the game's own word", 8, || {
            game.word_the_input_read_answered() & SHOOT != 0
        });
    });
}
