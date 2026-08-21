//! **What 妖々夢 answers a bot with**: its own names for its buttons, and the word one frame of a stage takes.
//!
//! Its own file and this game's own, for the reason each game's resume file is its own: which bit of a word is
//! this game's shot, where its player is, and what its own state byte says has just happened are questions
//! about *this* game. A shared answer could only be the union of every game's vocabulary, and 東方 is not one
//! kind of game to take a union of.
//!
//! **And nothing of a recipe is here**, `orb-bot` walking those: a bot is a harness rather than any part of
//! what orb does for somebody playing, so its own machinery is a crate outside the logic that ships. What is
//! left here is the two answers that need this game's addresses, which is what the game seam is for.
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

use super::{
    BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP, G_PLAYER, MENU_CANCEL, MENU_DECIDE,
    PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_HIT, PLAYER_STATE,
};

/// **The bit a recipe's button name is**, or `None` for a name 妖々夢 has nothing for.
///
/// The names are this game's, which is the line every other per-game thing in this tree is drawn on:
/// `Numbers` and `Reproduction` are 妖々夢's because their fields are, and these are because `decide` is a
/// shot and a return key here and something else elsewhere.
pub fn bit(name: &str) -> Option<u16> {
    Some(match name {
        "shot" => BUTTON_SHOT,
        "bomb" => BUTTON_BOMB,
        "focus" => BUTTON_FOCUS,
        "skip" => BUTTON_SKIP,
        "up" => PAD_UP,
        "down" => PAD_DOWN,
        "left" => PAD_LEFT,
        "right" => PAD_RIGHT,
        // The two masks every screen of this game is driven with — see [`Th07::menu_decide`].
        "decide" => MENU_DECIDE,
        "cancel" => MENU_CANCEL,
        _ => return None,
    })
}

/// The word a frame of a stage is played with.
///
/// # Safety
/// Must run on the game's main thread.
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
