//! **A person's pad, its focus held** — the word a sampled pad makes, read through the game's own `th06.cfg`
//! rather than through a table of orb's.
//!
//! A pad whose focus is bound to a button orb's own table did not name — a person's, on 2026-11-01, whose focus
//! is its L1 — pressed it and the word had no focus bit at all: the player did not slow down. What the game's
//! own read does with a pad button is what the answer has to do, and that is `the_buttons_pressed_through`'s
//! inverse.
//!
//! **The focus is the one of the five that is not simply a button that goes into the word**: the game holds the
//! player still off the count of frames a held button has been down (see `th06c`'s own `BUTTONS`), so a focus
//! that never reaches the word is a player that never slows.

use std::sync::atomic::Ordering;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_sim::button as xinput;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// Where the machine's own pad is.
const SLOT: u32 = 0;

/// How long orb's own sampling thread is given to read the pad, in frames with a millisecond of real time
/// each.
const WAITS: u32 = 3000;

/// The word's bit for the focus, which is what slows the player — `Th06c`'s `button::FOCUS`.
const FOCUS: u32 = 0x4;

/// **The button a person's own `th06.cfg` binds the focus to**, held — the left shoulder, as the person who
/// reported this has it, and written into the laid-out game's configuration so that the game's own binding is
/// the one being read and not the file it came with.
const THE_FOCUS_BUTTON: u16 = xinput::LEFT_SHOULDER;

/// The configuration's own number for that button: **XInput's bit index**, which is what a `th06.cfg` keeps a
/// pad button as — the left shoulder's `0x0100` is bit 8.
const THE_FOCUS_IN_THE_CONFIG: i16 = 8;

/// A launch nobody is playing, with the machine's pad in the first slot holding the focus.
fn a_person_holding_the_focus() -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        "th06c-a-pad-focus",
        Work::wandering(WORK_US),
        |config| config.autoplay = None,
    );
    // **The game's own binding written before it reads anything**: a person's file says which button its focus
    // is, and this one says the left shoulder.
    game.image().binds_the_pad_focus_to(THE_FOCUS_IN_THE_CONFIG);
    game.sim().xinput().attach(SLOT);
    game.sim().xinput().pushes(SLOT, THE_FOCUS_BUTTON);
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

/// **The focus a person binds reaches the game's own pad answer**, which is what slows the player.
#[test]
fn the_focus_a_person_binds_reaches_the_game() {
    in_its_own_process(|| {
        let game = a_person_holding_the_focus();
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            FOCUS,
            "the focus the game's own bindings name did not reach the game's pad answer:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And the wildcard import keeps the harness quiet about a constant it does not read.
        let _ = Ordering::Relaxed;
    });
}
