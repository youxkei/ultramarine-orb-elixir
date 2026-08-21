//! **orb playing 妖々夢 itself**: the recipe it gets through the screens with, and the word one frame of a
//! stage is played with.
//!
//! Its own file and 妖々夢's own rules, for the reason each game's resume file is its own: what a frame is
//! played with is a question about *this* game — which bit of a word is its shot, where its player is, what
//! its own state byte says has just happened to it. A shared bot could only be the union of every game's
//! vocabulary, and 東方 is not one kind of game to take a union of. If a second game ever wants one it writes
//! its own, and the day two of them turn out to want the same rule is the day to lift it out.
//!
//! **A run has to be started before it can be played, and starting one is menus.** That is not a rule about
//! playing — it is a sequence somebody carries out once — so it is written down rather than worked out:
//! `Config::autoplay` is the text of a recipe, and a recipe is a list of steps, each a word to hold and the
//! frames to hold it for. What a screen does with the shot held for two frames is the screen's business, so a
//! recipe can drive any of them without orb knowing what any of them are.
//!
//! **Then the rules**, which are Normal's, where the bullets are thin enough that a run does not have to
//! dodge to live:
//!
//! - the shot held on every frame
//! - the bomb on the frame the player is hit, which is 決死結界 — the death handler runs out of the *update*
//!   that finds [`super::PLAYER_STATE`] at 2, so a bomb pressed while it stands is a death taken back
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`, and `the_recipe_a_bot_plays_to` for the steps.

use std::sync::Mutex;

use orb_api::mem;

use super::{
    BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP, G_PLAYER, MENU_CANCEL, MENU_DECIDE,
    PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_HIT, PLAYER_STATE,
};
use crate::log;

/// The recipe this launch is playing to, and where it has got to in it — or `None` for a launch nobody asked
/// to play itself, and for one whose recipe could not be read.
///
/// A `Mutex` rather than a `MainThread`: what reads it is the game's own input read, which is the game's main
/// thread, but what *writes* it is [`settle`] at the attach — and the attach is not that thread on every
/// launch. Uncontended either way, one lock a frame being nothing beside a frame.
static RECIPE: Mutex<Option<Recipe>> = Mutex::new(None);

/// The steps left to press, and how many frames of the one in front are left.
struct Recipe {
    steps: Vec<Step>,
    /// Where in `steps` the one being held is, and the frames of it still to hold. A step is dropped by
    /// moving this on rather than by taking it out of the list, so that a log line can say which step of how
    /// many is being pressed.
    at: usize,
    held: u32,
}

/// One step of it: a word, and the frames to hold it for.
struct Step {
    word: u16,
    frames: u32,
}

/// **The bit a step's button name is**, or `None` for a name 妖々夢 has nothing for.
///
/// The grammar of a recipe is `orb_config::recipe`'s — a list of names and a count of frames, which is the
/// same list whatever game reads it — and the *names* are this game's. Which is the line every other
/// per-game thing in this tree is drawn on: `Numbers` and `Reproduction` are 妖々夢's because their fields
/// are, and these are 妖々夢's because `decide` is a shot and a return key here and something else
/// elsewhere.
fn bit(name: &str) -> Option<u16> {
    Some(match name {
        "shot" => BUTTON_SHOT,
        "bomb" => BUTTON_BOMB,
        "focus" => BUTTON_FOCUS,
        "skip" => BUTTON_SKIP,
        "up" => PAD_UP,
        "down" => PAD_DOWN,
        "left" => PAD_LEFT,
        "right" => PAD_RIGHT,
        // The two masks every screen of this game is driven with — see `Th07::menu_decide`.
        "decide" => MENU_DECIDE,
        "cancel" => MENU_CANCEL,
        _ => return None,
    })
}

/// Reads the recipe this launch is to play to, and says what it came to.
///
/// **A recipe that cannot be read is a launch that plays nothing**, rather than one that presses whatever a
/// half-read file came to: a step naming a button 妖々夢 has not is a recipe written for another game, and
/// pressing the rest of it would be driving somebody's screens by guesswork.
///
/// Said out loud either way. A launch that plays itself is one nobody is watching, so the log is the only
/// place it can be seen at all.
pub fn settle(recipe: Option<&str>) -> bool {
    let mut held = RECIPE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(text) = recipe else {
        *held = None;
        return false;
    };
    let written = match orb_config::recipe::read(text) {
        Ok(written) => written,
        Err(why) => {
            log!("autoplay: the recipe cannot be read: {why}");
            *held = None;
            return false;
        }
    };
    let mut steps = Vec::with_capacity(written.len());
    for step in &written {
        let mut word = 0;
        for name in &step.press {
            let Some(bit) = bit(name) else {
                log!("autoplay: the recipe cannot be read: 妖々夢 has no button called {name}");
                *held = None;
                return false;
            };
            word |= bit;
        }
        steps.push(Step {
            word,
            frames: step.frames,
        });
    }
    log!(
        "autoplay: orb is playing this run itself, to {} step(s)",
        steps.len()
    );
    *held = Some(Recipe {
        steps,
        at: 0,
        held: 0,
    });
    true
}

/// The word this frame is played with, or `None` for a launch that is nobody's to play.
///
/// The recipe first and the rules after it: every step is a screen on the way into a run, so the frames they
/// take are frames no rule about playing has anything to say about.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn word() -> Option<u16> {
    let mut held = RECIPE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let recipe = held.as_mut()?;
    if let Some(step) = recipe.steps.get(recipe.at) {
        recipe.held += 1;
        let word = step.word;
        if recipe.held >= step.frames {
            recipe.at += 1;
            recipe.held = 0;
            log!(
                "autoplay: step {} of {} pressed",
                recipe.at,
                recipe.steps.len()
            );
        }
        return Some(word);
    }
    Some(unsafe { playing() })
}

/// And the word a frame of a stage is played with, once the recipe has run out.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn playing() -> u16 {
    let mut word = BUTTON_SHOT;
    // **The frame the player is hit and not the frames after it.** 0x43ee0e leaves the state at 2 and the
    // update calls the death handler at 0x441fee where it finds it, so the bomb has to go in while that
    // stands — a frame later the deaths and the lives have already moved and a bomb is a bomb spent.
    if unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } == PLAYER_HIT {
        word |= BUTTON_BOMB;
    }
    word
}
