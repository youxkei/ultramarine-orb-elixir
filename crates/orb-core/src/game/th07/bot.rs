//! **orb playing 妖々夢 itself**: the word one frame of a stage is played with, worked out from what the game
//! holds.
//!
//! Its own file and 妖々夢's own rules, for the reason each game's resume file is its own: what a frame is
//! played with is a question about *this* game — where its player is, what its items are worth going for, what
//! its own state byte says has just happened to it. A shared bot could only be the union of every game's
//! vocabulary, and 東方 is not one kind of game to take a union of. If a second game ever wants one it writes
//! its own, and the day two of them turn out to want the same rule is the day to lift it out.
//!
//! **The rules are Normal's**, where the bullets are thin enough that a run does not have to dodge to live:
//!
//! - the shot held on every frame
//! - the bomb on the frame the player is hit, which is 決死結界 — the death handler runs out of the *update*
//!   that finds [`super::PLAYER_STATE`] at 2, so a bomb pressed while it stands is a death taken back
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`.

use orb_api::mem;

use super::{BUTTON_BOMB, BUTTON_SHOT, G_PLAYER, PLAYER_HIT, PLAYER_STATE};

/// The word this frame of a stage is played with.
///
/// # Safety
/// Must run on the game's main thread, with a stage running.
pub unsafe fn word() -> u16 {
    let mut word = BUTTON_SHOT;
    // **The frame the player is hit and not the frames after it.** 0x43ee0e leaves the state at 2 and the
    // update calls the death handler at 0x441fee where it finds it, so the bomb has to go in while that
    // stands — a frame later the deaths and the lives have already moved and a bomb is a bomb spent.
    if unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } == PLAYER_HIT {
        word |= BUTTON_BOMB;
    }
    word
}
