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
//! **The rules are Normal's**, and dodging comes before all of them:
//!
//! - the shot held on every frame, and [`BUTTON_SKIP`] while a dialogue is up
//! - the bomb the frame the player is hit, which is 決死結界 — 0x440cf0 counts a window down while
//!   [`super::PLAYER_STATE`] stands at 2, and a bomb inside it is a death taken back
//! - **a bullet within [`DANGER`] and the frame is spent leaving**: the safest of the nine one-frame moves,
//!   scored by how near the nearest bullet gets to where each lands
//! - and with nothing near, the way to a power item — the big one over a small one wherever both are up — or
//!   the boss's own x with the y low, or the middle of the field low
//!
//! **Dodging is the first of them because a bomb is a hit that was not stepped out of.** A launch that played
//! stage 1 without it spent two of its three bombs on 決死結界 by the midboss, and every one of those was a
//! frame with somewhere to stand. So the two are not blended — inside [`DANGER`] the bullets are the whole of
//! the decision and outside it they are not read at all, a blend being what walks into a bullet for an item.
//!
//! Nothing here presses for a 雑魚: the shot is held on every frame already, and what a killed one drops falls
//! to whoever is under it.
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`.

use orb_api::mem;

use super::{
    BULLET_ALIVE, BULLET_BYTES, BULLET_POSITION, BULLET_SLOTS, BULLET_STEP, BULLETS, BUTTON_BOMB,
    BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP, FIELD_HEIGHT, FIELD_LEFT, FIELD_TOP, FIELD_WIDTH,
    G_BULLET_MANAGER, G_GAME_MANAGER, G_ITEM_MANAGER, G_PLAYER, G_SUPERVISOR, GAME_MENU_UP,
    ITEM_ALIVE, ITEM_BIG_POWER, ITEM_BYTES, ITEM_KIND, ITEM_POSITION, ITEM_POWER, ITEM_SLOTS,
    MENU_CANCEL, MENU_DECIDE, PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_HIT, PLAYER_POSITION,
    PLAYER_STATE, SCENE_RUN, SUPERVISOR_SCENE, Th07,
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

/// How far off the field's bottom edge the player waits with nothing to go to, in the game's own units.
///
/// **A rule's own number and not a measurement of anything.** Against the edge outright is where the clamp at
/// 0x43f474 holds the player with the whole of one side taken away from it, and this much up leaves the room a
/// dodge needs on every side of it.
const ABOVE_THE_BOTTOM: f32 = 48.0;

/// How near a bullet has to be for the frame to be spent leaving rather than on where it was going.
///
/// **A rule's own number too.** Every bomb this bot spends is a hit it could have stepped out of, so the band
/// is wide enough to leave in — and no wider, a bot that dodged the whole screen being one that never collects
/// anything.
const DANGER: f32 = 40.0;

/// How far a candidate move is probed, which is a length rather than the step the game would really take: what
/// it is for is telling nine directions apart, and the frame's own step is three indirections whose one wrong
/// pointer is a bot that never moves — see [`CLOSE_ENOUGH`].
const A_PROBE: f32 = 8.0;

/// The nine one-frame moves, as the word each is and the direction it goes in.
///
/// **Standing still is first**, which is what settles a tie: where two moves are equally safe the bot holds its
/// ground rather than stepping back and forth between them every frame.
const MOVES: [(u16, f32, f32); 9] = [
    (0, 0.0, 0.0),
    (PAD_UP, 0.0, -1.0),
    (PAD_DOWN, 0.0, 1.0),
    (PAD_LEFT, -1.0, 0.0),
    (PAD_RIGHT, 1.0, 0.0),
    (PAD_UP | PAD_LEFT, -DIAGONAL, -DIAGONAL),
    (PAD_UP | PAD_RIGHT, DIAGONAL, -DIAGONAL),
    (PAD_DOWN | PAD_LEFT, -DIAGONAL, DIAGONAL),
    (PAD_DOWN | PAD_RIGHT, DIAGONAL, DIAGONAL),
];

/// A diagonal's own share of a step, which is the game's: 0x43f165 loads the speed's `+0x2c` for those where a
/// straight move takes `+0x24`, and what a corner of the unit square is is the same number either way.
const DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;

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
    // **The frame the player is hit and not the frames after it.** 0x43edc0 leaves the state at 2 and zeroes
    // the timer, and 0x440cf0 counts `[player+0x23f8]` down from there while the state stands — a window this
    // presses into rather than a single frame, and one whose end takes the life.
    if unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } == PLAYER_HIT {
        word |= BUTTON_BOMB;
    }
    // A dialogue read at the game's own pace is the afternoon this was written to save.
    if unsafe { Th07::dialogue_line() }.is_some() {
        word |= BUTTON_SKIP;
    }
    let (steer, dodging) = unsafe { steer() };
    // **低速移動 under a boss**, which is where this game's shots do their damage — and off again the moment
    // there is something to dodge, the same key halving the step a dodge is made of.
    if !dodging && unsafe { Th07::boss() }.is_some() {
        word |= BUTTON_FOCUS;
    }
    word | steer
}

/// **The 十字キー of the safest of the nine one-frame moves**, or the one towards where the rules want to be
/// where nothing is near enough to dodge.
///
/// The safety of a move is how near the nearest bullet gets to where that move lands — so the choice covers
/// every direction at once, and up and down are dodges as much as left and right. Two regimes and not a blend,
/// because a blend is a bot that walks into a bullet for an item: inside [`DANGER`] the frame is the dodge, and
/// outside it the bullets are not read at all.
///
/// **Lasers are not in this.** They are their own manager's slots — [`super::G_LASERS`] — and a length and an
/// angle rather than a point, so nothing here would make them a distance; a launch that dies on one is what
/// says whether that matters.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn steer() -> (u16, bool) {
    let at = unsafe { player() };
    let field = unsafe { field() };
    let wanted = unsafe { where_to_be(field) };
    // Where each move lands, held inside the field the way the clamp at 0x43f416 would hold it: a move into a
    // wall is a move that does not happen, and scoring it as though it did would pick it.
    let lands = MOVES.map(|(_, dx, dy)| {
        (
            (at.0 + dx * A_PROBE).clamp(field.0, field.0 + field.2),
            (at.1 + dy * A_PROBE).clamp(field.1, field.1 + field.3),
        )
    });
    // One walk of the slots and nine running minima out of it, rather than nine walks. Squared throughout: what
    // is wanted is which is nearest and not how near, and a square root a slot is a square root wasted.
    let mut nearest = [f32::INFINITY; MOVES.len()];
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        if unsafe { mem::read::<u16>(bullet + BULLET_ALIVE) } == 0 {
            continue;
        }
        // Where it will be and not where it is: its own update adds its step before the frame this word is
        // played on, so the bullet a dodge has to miss is the one a frame ahead.
        let (bx, by) = unsafe { position(bullet + BULLET_POSITION) };
        let (sx, sy) = unsafe { position(bullet + BULLET_STEP) };
        let (bx, by) = (bx + sx, by + sy);
        for (near, (x, y)) in nearest.iter_mut().zip(lands) {
            *near = near.min((x - bx).powi(2) + (y - by).powi(2));
        }
    }
    if nearest[0] >= DANGER * DANGER {
        return (unsafe { towards(wanted) }, false);
    }
    // The **first** of the safest and not the last, which is what [`MOVES`]'s own order is for: `max_by` hands
    // back the last of a tie, so standing still would lose every one of them to a diagonal and the bot would
    // step between two equally safe places every frame.
    let word = MOVES
        .iter()
        .zip(nearest)
        .fold(None, |best, ((word, _, _), near)| match best {
            Some((_, safest)) if near <= safest => best,
            _ => Some((*word, near)),
        })
        .map_or(0, |(word, _)| word);
    (word, true)
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
unsafe fn where_to_be(field: (f32, f32, f32, f32)) -> (f32, f32) {
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // A power item at the place it actually is, the y as much as the x, so one high up the field is one the bot
    // goes up for. Under a boss it is that boss's x with the y low, a fight being a thing to stand under rather
    // than to reach; and with neither it is the middle, low — the shortest way to wherever the next one falls.
    if let Some(item) = unsafe { nearest_power_item() } {
        return item;
    }
    match unsafe { Th07::boss() } {
        Some(boss) => (
            unsafe { mem::read::<f32>(boss + super::ENEMY_POSITION) },
            low,
        ),
        None => (field.0 + field.2 / 2.0, low),
    }
}

/// The power item worth going for, or `None` where there is none.
///
/// **A big one beats a small one wherever both are on the screen**, whatever the distances: 妖々夢's
/// 大パワーアイテム is eight steps of power against the small one's one, so it is worth crossing the field for
/// where the small one is worth a step aside. Between two of a kind it is the nearer.
///
/// **Every slot walked and not the live count taken**, because the count says how many there are and not
/// which slots they are in: the manager leaves a collected item's own numbers where they were and clears
/// [`ITEM_ALIVE`], which is the same walk 0x432a3f makes.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn nearest_power_item() -> Option<(f32, f32)> {
    let (at_x, at_y) = unsafe { player() };
    (0..ITEM_SLOTS)
        .map(|slot| G_ITEM_MANAGER + slot * ITEM_BYTES)
        .filter(|item| unsafe { mem::read::<i8>(item + ITEM_ALIVE) } != 0)
        .filter_map(|item| {
            let worth = match unsafe { mem::read::<i8>(item + ITEM_KIND) } {
                ITEM_BIG_POWER => 1,
                ITEM_POWER => 0,
                _ => return None,
            };
            let at = unsafe { position(item + ITEM_POSITION) };
            Some((worth, (at.0 - at_x).powi(2) + (at.1 - at_y).powi(2), at))
        })
        // The worth first and the distance only inside it, which is what makes a big one across the field beat
        // a small one under the player's hand.
        .max_by(|(ours, ourm, _), (theirs, theirm, _)| {
            ours.cmp(theirs).then(theirm.total_cmp(ourm))
        })
        .map(|(_, _, at)| at)
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
