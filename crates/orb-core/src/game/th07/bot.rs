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
//! - the y held [`ABOVE_THE_BOTTOM`] off the field's bottom edge, always
//! - the x held on the boss under a boss, on the nearest power item where one is on the screen, and in the
//!   middle of the field with neither
//!
//! **The y is one rule and not two** — held low through the way as well as through a fight — because a power
//! item falls: 0x432d27 steps its own y down every update, so a run that lines its x up and stays low is
//! handed the item where it is safe to be, while one that climbed to meet it would be up among the bullets
//! for nothing. Which is also why nothing here presses for a 雑魚: the shot is held on every frame already,
//! and what a killed one drops falls to whoever is under it.
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`.

use orb_api::mem;

use super::{
    BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP, FIELD_HEIGHT, FIELD_LEFT, FIELD_TOP,
    FIELD_WIDTH, G_GAME_MANAGER, G_ITEM_MANAGER, G_PLAYER, G_SUPERVISOR, GAME_MENU_UP, ITEM_ALIVE,
    ITEM_BYTES, ITEM_KIND, ITEM_POSITION, ITEM_POWER, ITEM_SLOTS, MENU_CANCEL, MENU_DECIDE,
    PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_HIT, PLAYER_POSITION, PLAYER_STATE, SCENE_RUN,
    SUPERVISOR_SCENE, Th07,
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

/// How far off the field's bottom edge the player is held, in the game's own units.
///
/// **A rule's own number and not a measurement of anything.** Off the edge outright is where the clamp at
/// 0x43f474 would hold the player against a wall it cannot read, and far enough up to be worth dodging in is
/// where the y would have to be read against the bullets — which is a rule this bot does not have.
const ABOVE_THE_BOTTOM: f32 = 48.0;

/// How near the place it wants to be is near enough to press nothing.
///
/// **Wide enough that a frame's own step cannot cross it**, which is what stops the word flipping between
/// [`PAD_LEFT`] and [`PAD_RIGHT`] every frame around a target the player is standing on. The frame's step
/// itself would be the exact bound and is not read: it is `[[player+0xb7e70]+0x24]` for a straight move and
/// `+0x2c` for a diagonal, times `[player+0x23f0]` and `[0x575ac8]` — three indirections whose one wrong
/// pointer is a bot that never moves at all, against a band too wide costing a few units of accuracy the
/// item's own collect radius covers anyway.
const CLOSE_ENOUGH: f32 = 8.0;

/// The word the rules press this frame.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn word_to_press() -> u16 {
    // **Nothing at all off a stage, the shot included.** The rules are asked for a word by the game's own
    // keyboard read, which every screen makes — and a menu of this game's is taken by the shot as much as by
    // the return key, so a word held on the title screen walks that menu once and then holds it, while
    // 十字キー there moves a cursor these rules cannot read. The front end is the recipe's to drive, one press
    // to a screen; a rule pressing alongside it is a press nobody wrote.
    if !unsafe { in_a_stage() } {
        return 0;
    }
    let mut word = BUTTON_SHOT;
    // **The frame the player is hit and not the frames after it.** 0x43ee0e leaves the state at 2 and the
    // update calls the death handler at 0x441fee where it finds it, so the bomb has to go in while that
    // stands — a frame later the deaths and the lives have already moved and a bomb is a bomb spent.
    if unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } == PLAYER_HIT {
        word |= BUTTON_BOMB;
    }
    let wanted = unsafe { where_to_be() };
    word | unsafe { towards(wanted) }
}

/// Whether a stage is running to be steered in.
///
/// **The supervisor's own scene and not the run's numbers being there**, which is what this was and what it
/// cost: a launch reached the title screen with that pointer still set, so the rules ran with the play field
/// never filled in — a field of zeroes puts the y they want above its own top, and the [`PAD_UP`] that went in
/// wrapped 妖々夢's title menu from its first item round to 終了, which the shot beside it then took. The scene
/// is the same thing `Th07::read_state` answers `playing` out of, and it is a run's on no other screen.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_stage() -> bool {
    let scene = unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_SCENE) };
    let paused = unsafe { mem::read::<u8>(G_GAME_MANAGER + GAME_MENU_UP) };
    scene == SCENE_RUN && paused == 0
}

/// Where the rules want the player: the x whatever is worth being under has, and the y always low.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn where_to_be() -> (f32, f32) {
    let field = unsafe { field() };
    let x = match unsafe { Th07::boss() } {
        Some(boss) => unsafe { mem::read::<f32>(boss + super::ENEMY_POSITION) },
        // The nearest power item, or the middle of the field where there is none — a place to wait that is
        // the shortest way to wherever the next one falls.
        None => unsafe { nearest_power_item() }.unwrap_or(field.0 + field.2 / 2.0),
    };
    (x, field.1 + field.3 - ABOVE_THE_BOTTOM)
}

/// The x of the nearest power item on the screen, or `None` where none is.
///
/// **Every slot walked and not the live count taken**, because the count says how many there are and not
/// which slots they are in: the manager leaves a collected item's own numbers where they were and clears
/// [`ITEM_ALIVE`], which is the same walk 0x432a3f makes.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn nearest_power_item() -> Option<f32> {
    let (at_x, at_y) = unsafe { player() };
    (0..ITEM_SLOTS)
        .map(|slot| G_ITEM_MANAGER + slot * ITEM_BYTES)
        .filter(|item| unsafe { mem::read::<i8>(item + ITEM_ALIVE) } != 0)
        .filter(|item| unsafe { mem::read::<i8>(item + ITEM_KIND) } == ITEM_POWER)
        .map(|item| unsafe { position(item + ITEM_POSITION) })
        .map(|(x, y)| ((x - at_x).powi(2) + (y - at_y).powi(2), x))
        .min_by(|(near, _), (far, _)| near.total_cmp(far))
        .map(|(_, x)| x)
}

/// The 十字キー bits that take the player towards `wanted`, and nothing for an axis it is already on.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn towards((x, y): (f32, f32)) -> u16 {
    let (at_x, at_y) = unsafe { player() };
    let mut word = 0;
    if x - at_x > CLOSE_ENOUGH {
        word |= PAD_RIGHT;
    } else if at_x - x > CLOSE_ENOUGH {
        word |= PAD_LEFT;
    }
    // The y grows downwards, which is the direction the clamp at 0x43f474 reads its edges in: `top` is the
    // smaller of the two.
    if y - at_y > CLOSE_ENOUGH {
        word |= PAD_DOWN;
    } else if at_y - y > CLOSE_ENOUGH {
        word |= PAD_UP;
    }
    word
}

/// Where the player is.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player() -> (f32, f32) {
    unsafe { position(G_PLAYER + PLAYER_POSITION) }
}

/// The two floats a position is, wherever one is kept.
///
/// # Safety
/// Must run on the game's main thread, with `at` the address of a position.
unsafe fn position(at: usize) -> (f32, f32) {
    (unsafe { mem::read::<f32>(at) }, unsafe {
        mem::read::<f32>(at + size_of::<f32>())
    })
}

/// The play field, as its left, top, width and height.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    (
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_LEFT) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_TOP) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_WIDTH) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_HEIGHT) },
    )
}
