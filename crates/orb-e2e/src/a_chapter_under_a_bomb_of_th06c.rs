//! **A bomb of th06c is not a frame orb keeps away from either**, and the chapter that begins under one says
//! so.
//!
//! `chapter::guarded` is the list of frames no chapter may begin on — mid-dialogue and paused — and a bomb is
//! deliberately not on it, for the reason its comment records and `a_chapter_under_a_bomb.rs` holds th06 to.
//! What this game brings of its own is the flag: th06c keeps the bomb in use in its own player —
//! `player::BOMB_IN_USE` — and orb reads a bomb off that word and off nothing else. So a boundary the fight
//! really has begins its chapter whatever the player has just spent, and the flag turns up in the line that
//! carries the frame's state — which is where somebody looking into a boundary they cannot account for reads
//! a bomb back rather than blaming the frame.
//!
//! **The bomb is a deathbomb**, the one this laid-out game has: X held while the frames a bomb is still
//! taken in run down after a hit, which is the real one's own window — `+0x7254` counted down at
//! `0x14004a7ae` — and the bomb the rules a bot plays by spend and no other. Pressed at the keyboard, the
//! way somebody playing presses it, rather than the image being told the flag is up: the flag this reads
//! back is then the one the game's own bomb raised.

use crate::fake::th06c::{A_BOMB_LASTS, ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage frame the hit the bomb takes back is taken on.
///
/// Far enough before [`ATTACK_CHANGES`] that the boundary falls inside the bomb's own frames, and far enough
/// after it that the bomb has not run out by then: the whole point is a chapter beginning while the flag is
/// up.
const HIT_AT: u32 = ATTACK_CHANGES - A_BOMB_LASTS / 2;

#[test]
fn a_chapter_of_a_fight_of_th06c_begins_while_a_bomb_is_going_off() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-a-bomb",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(0);

        // The card's chapter, which is the one the bomb is spent inside: the fight's own boundaries are found
        // as it is fought, and the next of them is the attack after this card.
        game.frames_until("the card's chapter", MIDBOSS_BEATEN, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });

        // The hit, a little before the attack changes, and X held through the frames a bomb is still taken
        // in: one bomb fewer than the run had, the flag orb reads one by up in the game's own memory, and no
        // death — the bomb took it back.
        game.frames_until("the frame the hit is taken on", MIDBOSS_BEATEN, || {
            game.state().stage_frames >= HIT_AT
        });
        let bombs_before = game.state().bombs;
        game.keyboard().set(keys::X, true);
        game.hit();
        game.frames_until("the bomb going off", A_BOMB_LASTS, || game.state().bombing);
        game.keyboard().set(keys::X, false);
        let state = game.state();
        assert_eq!(state.bombs, bombs_before - 1, "the bomb was not spent");
        assert!(
            state.stage_frames < ATTACK_CHANGES,
            "the bomb went off after the attack it was to be under had changed: {state}",
        );

        // And the boundary the fight has there begins its chapter all the same, on the frame the attack
        // changed rather than on the first frame after the bomb.
        game.frames_until("the chapter the attack change is", A_BOMB_LASTS, || {
            log.said(&format!(
                "chapter 4 at frame {ATTACK_CHANGES} (script {ATTACK_CHANGES}): a midboss nonspell"
            ))
        });
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
        game.frames_until("a state line written under the bomb", A_BOMB_LASTS, || {
            log.lines()
                .iter()
                .any(|line| line.contains(" bombing clears="))
        });

        // The bomb ends, and the run goes on: what the flag is worth depends on its going out again, and a
        // chapter is due for no reason of the bomb's on the frame it does.
        game.frames_until("the bomb over", A_BOMB_LASTS, || !game.state().bombing);
        let after = game.state().stage_frames;
        let ended = log.written();
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            after + 10,
            "the game stopped being played when the bomb ended",
        );
        assert!(
            !log.said_since(ended, "chapter 5 at frame"),
            "a bomb ending began a chapter of its own:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("died in chapter"),
            "the hit the bomb was pressed inside was taken as a death:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
