//! **One pad on two interfaces, which is every pad on a machine that has XInput.**
//!
//! An XInput pad is also a winmm device, so orb's own sweep finds the same physical pad twice — once as a
//! winmm index and once as an XInput slot. Both were candidates for `joystick::last_pushed`, and both are
//! pushed by one press, so the choice went to whichever of the two reported it later.
//!
//! **What that does to the player is what a session measured.** A run of th07 played on a pad that showed
//! up as `winmm 1` (mid=1532 pid=0a57) and `xinput 0` had the log write
//! `is the pad the run is played with` eighteen times — nine `winmm 1` → `xinput 0` pairs, forty
//! milliseconds apart, which is one press seen twice with the two interfaces' own delays between them. And a
//! press whose socket stops being sampled halfway through it is a press the game is not told about: the
//! player pushed the d-pad and did not move, let go and moved further.
//!
//! So a pad that answers on XInput is read there and its winmm twin is put down. What that costs is a pad
//! XInput cannot see at all, which is the only thing winmm is left for.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;

/// The pad, as an Xbox One pad answers `joyGetDevCapsA`: sixteen buttons and a Y axis over the whole of a
/// 16-bit travel. In winmm's second socket, which is where a machine whose joystick 0 is something else
/// leaves it.
const BUTTONS: u32 = 16;
const TRAVEL: (u32, u32) = (0, 65535);
const WINMM: u32 = 1;
const XINPUT: u32 = 0;

/// A button of the pad's, pushed to make orb choose: XInput's own A, and winmm's first button.
const A: u16 = 0x1000;
const FIRST: u32 = 1;

/// How long an e2e test gives orb's own sampling thread, in frames with a millisecond of real time each —
/// the same wait `mode_on_a_winmm_pad` gives it, and for the same reason: what is waited for is a thread
/// being scheduled rather than a number of the game's frames.
const WAITS: u32 = 3000;

fn waits_until(game: &Fake, what: &str, done: impl Fn() -> bool) {
    for _ in 0..WAITS {
        if done() {
            return;
        }
        game.frame();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!(
        "{what} did not happen in {WAITS} frame(s):\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// Which sockets orb has said the run is played with, in the order it said them.
fn chosen(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains("is the pad the run is played with"))
        .collect()
}

#[test]
fn a_pad_on_both_interfaces_is_the_run_played_with_on_xinput_alone() {
    in_its_own_process(|| {
        let game = Fake::attach("both-interfaces", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        // The one pad, plugged into both of the places it shows up in.
        game.sim()
            .joystick()
            .attach_at(WINMM, BUTTONS, TRAVEL.0, TRAVEL.1);
        game.sim().xinput().attach(XINPUT);
        game.at_the_title_menu();
        waits_until(&game, "orb's own thread reading the pad", || {
            !chosen(&game).is_empty()
        });

        // The press as the two interfaces deliver it, and in the order a session measured them in: the
        // winmm half of the pad first, then the XInput half forty milliseconds later. Frames between them,
        // so that orb's own thread has cycles to read the winmm half in if it is going to.
        game.sim().joystick().pushes_at(WINMM, FIRST, 0, 0);
        game.frames(60);
        game.sim().xinput().pushes(XINPUT, A);
        game.frames(60);
        assert!(
            game.log()
                .said("winmm is left alone while XInput answers for a pad"),
            "winmm was not put down for the pad XInput answers for:\n  {}",
            game.log().lines().join("\n  "),
        );

        let chose = chosen(&game);
        assert!(
            chose.iter().all(|line| line.contains("xinput 0")),
            "a pad XInput answers for was read on winmm as well:\n  {}",
            chose.join("\n  "),
        );
    });
}
