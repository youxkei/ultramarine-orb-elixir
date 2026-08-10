//! **A bomb is not a frame orb keeps away from**, and the chapter that begins under one says so.
//!
//! `chapter::guarded` is the list of frames no chapter may begin on — mid-dialogue and paused — and a
//! bomb is deliberately not on it. What its comment records is why: a bomb was guarded against on the
//! strength of a boundary seen where one had cleared a boss's bullets, and the detector cannot have
//! spoken there, since it asks for no enemies *and* no boss. So the guard came off, and what is left
//! standing is the claim this e2e test is: a boundary the fight really has begins its chapter whatever
//! the player has just spent.
//!
//! The other half of that comment is where a bomb is read back from — the lines that carry the state, so
//! that a boundary nothing accounts for can be blamed on the one signal rather than on the whole frame.
//! Which is what makes `Player::bombInUse` worth laying out at all: it is in no decision of orb's and in
//! every line orb writes about a frame.

use crate::fake::th06::{ATTACK_CHANGES, CARD_STARTS, Fake, SHAKE_FRAMES, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Screen;
use orb_sim::keys;

/// The stage frame the bomb goes off on.
///
/// Far enough before [`ATTACK_CHANGES`] that the boundary falls inside the bomb's own frames, and far
/// enough after it that the shake has not run out by then: the whole point is a chapter beginning while
/// the flag is up.
const BOMBS_AT: u32 = ATTACK_CHANGES - SHAKE_FRAMES as u32 / 2;

#[test]
fn a_chapter_of_the_fight_begins_while_a_bomb_is_going_off() {
    in_its_own_process(|| {
        let game = Fake::attach("a-bomb", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.at_the_title_menu();
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode chosen", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the shot type select", 90, || {
            let front = game.image().front_end_now();
            front.screen == Screen::ShotType && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames_until("the stage built", 8, || game.state().playing);

        // The card's chapter, which is the one the bomb is spent inside: the fight's own boundaries are
        // found as it is fought, and the next of them is the attack after this card.
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });

        // ボム, a few dozen frames before the attack changes. The bomb is spent — one fewer than the run
        // had — and the flag orb reads one by is up in the game's own memory.
        game.frames_until("the frame the bomb goes off on", BOMBS_AT + 60, || {
            game.state().stage_frames >= BOMBS_AT
        });
        let bombs_before = game.state().bombs;
        game.bombs();
        game.frame();
        let state = game.state();
        assert_eq!(state.bombs, bombs_before - 1, "the bomb was not spent");
        assert!(
            state.bombing,
            "the bomb orb reads is not going off: {state}",
        );

        // And the boundary the fight has there begins its chapter all the same, on the frame the attack
        // changed rather than on the first frame after the bomb.
        game.frames_until(
            "the chapter the attack change is",
            SHAKE_FRAMES as u32,
            || {
                log.said(&format!(
                "chapter 4 at frame {ATTACK_CHANGES} (script {ATTACK_CHANGES}): a midboss nonspell"
            ))
            },
        );
        assert_eq!(
            game.state().stage_frames,
            ATTACK_CHANGES,
            "the chapter began somewhere other than the frame the attack changed on",
        );
        assert!(
            game.state().bombing,
            "the bomb was over by the frame the chapter began, so nothing was begun under one",
        );

        // Said out loud in the line that carries the frame's state, which is where somebody looking into a
        // boundary they cannot account for reads a bomb back — see `chapter::guarded`.
        game.frames_until(
            "a state line written under the bomb",
            SHAKE_FRAMES as u32,
            || {
                log.lines()
                    .iter()
                    .any(|line| line.contains(" bombing clears="))
            },
        );

        // The bomb ends where its own shake does, and the run goes on: what the flag is worth depends on
        // its going out again, and a chapter is due for no reason of the bomb's on the frame it does.
        game.frames_until("the bomb over", SHAKE_FRAMES as u32 * 2, || {
            !game.state().bombing
        });
        let after = game.state().stage_frames;
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            after + 10,
            "the game stopped being played when the bomb ended",
        );
    });
}
