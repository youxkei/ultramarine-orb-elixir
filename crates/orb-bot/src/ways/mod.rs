//! **The autoplay engine's own ways**: for each game and difficulty, the cards, the fire after a card, the chapters
//! and the stretches of a stage the rules lost a run on, each with the way that took it — taken up wherever a run
//! of that difficulty is, in either mode, whatever the recipe it is played to.
//!
//! **Here and not in the recipes**, which is where they were first written: a way is what it takes to get through an
//! attack, and that is the same for every run that meets it — a whole run, a stage practised on its own, a chapter
//! taken again — so a recipe naming them had to name them again in every recipe that reached the attack, and a recipe
//! that did not was a run that lost it. What a recipe still says is where it differs: a way it names for the same
//! card, fire, chapter or stretch is taken in place of the engine's.
//!
//! **Rust and not a file in the recipe's grammar**, so that a way named for a card is the name of a way there is: a
//! table read at the attach would find a misspelled way the launch it was read in, after the rules had played on
//! without it.
//!
//! **A chapter where a way is a chapter's, and a stretch of the stage's own frames where it is narrower than one**:
//! a chapter is counted in either mode (`orb_core::chapter::Chapters::counts_only`), and th10's stage 4 wants five
//! ways inside its one chapter before 5136.

use crate::Ways;

mod th06c;
mod th06nc;
mod th08;
mod th10;

/// **The difficulties a way is written for, as every game here numbers them**: Easy 0, Normal 1, Hard 2, Lunatic 3,
/// and the Extra 4 — each game's own `DIFFICULTY_EXTRA`.
const NORMAL: i32 = 1;
const EXTRA: i32 = 4;

/// **The engine's ways for a run of the game orb calls `game` at `difficulty`**, and none where nothing is written
/// for it.
pub(crate) fn for_a_run_of(game: &str, difficulty: i32) -> Ways {
    match (game, difficulty) {
        ("th06c", EXTRA) => th06c::extra(),
        ("th06nc", EXTRA) => th06nc::extra(),
        ("th08", NORMAL) => th08::normal(),
        ("th08", EXTRA) => th08::extra(),
        ("th10", NORMAL) => th10::normal(),
        ("th10", EXTRA) => th10::extra(),
        _ => Ways::default(),
    }
}

/// **The floor, the x left to the rules**, which is where more than one game's Extra holds the run.
const ON_THE_FLOOR: crate::steer::Standing = crate::steer::Standing {
    across: None,
    y: Some(432.0),
};
