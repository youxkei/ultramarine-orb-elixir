//! **A person's pad, answered by orb at the game's own read** — the other half of `runtime::keyboard_state`'s
//! composition, and the reason `runtime::xinput_get_state` answers every launch.
//!
//! th06c looks `XInputGetState` up for itself, so a launch orb does not answer it in is a launch where the game
//! reads the pads behind orb's back: what orb answered a launch nobody is playing with was a pad at rest, and
//! the pad orb had sampled itself reached the game only as keys typed into its keyboard snapshot. That is one
//! press arriving twice — a keyboard read and a pad read are two edges out of it, and a menu moved its cursor on
//! both. The two are the same device, so what the game is answered with is orb's sample: the pad's own half of
//! the word orb decided over the keyboard and the pad together.
//!
//! **And a question of orb's holds the pad as it holds the keyboard.** This game's screens take their decide
//! off `A` and off the shot both, so those two bits are what leaves the answer while `HOLD_DECIDE` says so —
//! the same two [`orb_core::runtime::held_back`] takes out of the keyboard's half.
//!
//! **And it is the binding that is on trial as much as the press.** th06c decides once, in its startup,
//! whether it has an XInput pad at all, and reads the pad through that answer for the rest of the launch —
//! `fake::th06c`'s `Pad`, which asks the same question of the same read before its first frame. An answer of
//! `NOT_CONNECTED` there is a record bound to winmm, and a pad read by the game itself is a pad orb's hold
//! never reaches: this test's pad is in XInput alone, so a launch bound the other way never sees its press.

use std::sync::atomic::Ordering;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::runtime::HOLD_DECIDE;
use orb_sim::button as xinput;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// Where the machine's own pad is, which is the development machine's own slot.
const SLOT: u32 = 0;

/// How long orb's own sampling thread is given to read the pad, in frames with a millisecond of real time
/// each.
const WAITS: u32 = 3000;

/// What the game's own word holds for a pad's `A`: the shot the game's `th06.cfg` binds to it and the decide
/// its own button twelve is whatever is bound — see `fake::the_word_of_a_bound_pad`, which is `0x14000f710`'s
/// reading, and not orb's own numbering of the same buttons.
const A_PRESSES: u32 = 0x0001 | 0x0100;

/// A launch nobody is playing, with the machine's pad in the first slot holding `A` once orb has sampled it.
fn a_person_holding_a() -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        "th06c-a-persons-pad",
        Work::wandering(WORK_US),
        |config| config.autoplay = None,
    );
    game.sim().xinput().attach(SLOT);
    game.sim().xinput().pushes(SLOT, xinput::A);
    for _ in 0..WAITS {
        if game
            .log()
            .said(&format!("joystick: xinput {SLOT} has a pad"))
        {
            return game;
        }
        game.frame();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!(
        "orb's sampling never read the pad in xinput {SLOT}:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// **A pad that starts answering is the one played with, though nothing pushes it** — a controller swapped
/// mid-run is a new pad held still the moment it arrives, and the socket it replaces may go on answering with
/// the pad left where it was (reported 2026-11-01, "新しいコントローラーを使ってるからこれ全部のコントローラーを
/// 見てるはずだよね").
#[test]
fn a_pad_that_starts_answering_is_the_one_played_with() {
    in_its_own_process(|| {
        let game = a_person_holding_a();
        // **Another pad arrives while the old one still answers**, at rest where it was left: what picks it
        // is that it has started answering at all, not a push and not the old socket going quiet.
        game.sim().xinput().attach(1);
        for _ in 0..WAITS {
            if game
                .log()
                .said("joystick: xinput 1 is the pad the run is played with")
            {
                return;
            }
            game.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        panic!(
            "orb's sampling never played with the pad that arrived:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// **The pad's press reaches the game through the read orb answers it with**, where a launch nobody is playing
/// has no bot's word to fill that read with.
#[test]
fn a_persons_pad_is_answered_with_the_pad_orb_sampled() {
    in_its_own_process(|| {
        let game = a_person_holding_a();
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & A_PRESSES,
            A_PRESSES,
            "the pad's press never reached the game's own pad read:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And its decide does not, on a frame a question of orb's is up** — held out of the pad's answer as it is
/// out of the keyboard's, which is the one hold over both because the two are one word.
#[test]
fn a_persons_pad_is_held_back_while_a_question_of_orbs_is_up() {
    in_its_own_process(|| {
        let game = a_person_holding_a();
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & A_PRESSES,
            A_PRESSES,
            "the pad's press was not there to hold:\n  {}",
            game.log().lines().join("\n  "),
        );
        // The hold as the frame after a question going up reads it — see `runtime::draw_overlay`, which
        // writes it from the screen's own state at the end of every frame and so cannot be set before one.
        HOLD_DECIDE.store(true, Ordering::Relaxed);
        game.frames(1);
        HOLD_DECIDE.store(false, Ordering::Relaxed);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & A_PRESSES,
            0,
            "a question of orb's was up and the pad's decide reached the game as well:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
