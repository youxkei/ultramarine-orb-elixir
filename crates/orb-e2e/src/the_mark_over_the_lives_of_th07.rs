//! **The brush stroke over th07's count of lives, and the row it has to stay inside.**
//!
//! What the stroke *is* has tests already — `lives_ui.rs`'s own, and `a_pointdevice_run_of_th07.rs` reads
//! `DISABLE` off the screen over a run of this game. What is left, and what these are, is this game's own half
//! of it: **this panel has no room around that row**. th06 leaves 8 pixels of panel either side, which the
//! stroke's own box wants 7 of, and th07 puts the bombs 16 pixels under a row 16 tall — so the mark here has
//! the row and nothing else, and the strips th06's mark paints its panel tile across do not exist. That the
//! tile itself is the one out of the vm the game lays its background with is `Th07::panel_tile`'s own test.
//!
//! Every one of them reads what was drawn off the device, which is what somebody watching the screen saw.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th07::Th07;
use orb_sim::Quad;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The row the count of lives is in, as a quad to hold what was drawn against: `Th07::lives_row`'s own
/// rectangle, which is where the game paints that count and nothing of orb's may paint over.
fn lives_row() -> Quad {
    let row = Th07.lives_row();
    Quad {
        x: row.left,
        y: row.top,
        width: row.width,
        height: row.height,
        color: 0,
        texture: 0,
    }
}

#[test]
fn th07s_mark_is_one_word_over_the_count_with_no_panel_painted_beside_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-mark",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.in_a_pointdevice_run();
        game.one_frame();
        let log = game.log();

        // The tile orb found, said in the log because the difference is visible and was not: without the
        // game's own tile the strips either side of the stroke are a flat colour, which is exactly the patch
        // this is meant not to be.
        assert!(
            log.said("lives: the panel's own tile is what the strips are painted with"),
            "orb did not take the tile out of the vm the game lays its panel with:\n  {}",
            log.lines().join("\n  ")
        );

        // The word on the stroke, over the row the count is in.
        let word = game.says("DISABLE");
        assert_eq!(word.len(), 1, "the mark is not one word on the screen");
        assert!(
            lives_row().overlaps(&word[0]),
            "the word is not over the row the count is in",
        );

        // And **nothing painted with that tile**, which is this game's own answer and not an omission: the
        // strips are the panel the stroke reaches past the row into, and th07 leaves none to reach — its
        // bombs are 16 pixels under a row 16 tall, so the mark has the row and the tile has nothing to paint.
        // What the reading is for is the two strips th06 does have, and `the_mark_over_the_lives.rs` is where
        // they are laid tile by tile.
        let (texture, _) = game.image().the_panels_tile();
        let painted: Vec<_> = game
            .drawn()
            .quads
            .into_iter()
            .filter(|quad| quad.texture == texture)
            .collect();
        assert!(
            painted.is_empty(),
            "the panel was painted where this game leaves no room to paint it: {painted:?}",
        );
    });
}

/// And **nothing of the mark reaches the rows either side of it**, which for this game means the row itself and
/// no more: th07 puts the bombs 16 pixels under a row 16 tall — the same loop from y at 0x498c54 (96.0) and
/// 0x498ca8 (112.0) — so there is no panel between the two to reach into.
///
/// Watched on a launch: the stroke's own box is 30 tall against a row of 16, so it hung 7 pixels into the bombs'
/// row, the strip below it was painted with the panel's own tile, and the tops of the bombs' stars went with it.
/// th06 has 8 pixels of panel there and the same box fits inside them, which is why this is a question about
/// this game.
#[test]
fn nothing_of_th07s_mark_reaches_the_rows_either_side_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-mark-inside-its-row",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.in_a_pointdevice_run();
        game.one_frame();

        let row = Th07.lives_row();
        let outside: Vec<_> = game
            .drawn()
            .quads
            .into_iter()
            .filter(|quad| quad.y < row.top || quad.y + quad.height > row.top + row.height)
            .collect();
        assert!(
            outside.is_empty(),
            "the mark reached the panel rows either side of the count's own: {outside:?}",
        );
    });
}

/// And a panel orb cannot find a tile in is painted flat, with the log saying so rather than leaving somebody
/// to wonder why the panel has a patch in it: which is every frame before the game's own build has put a
/// sprite on that vm.
#[test]
fn a_panel_with_no_tile_of_its_own_is_painted_flat_and_said_to_be() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-mark-no-tile",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.has_no_panel_tile();
        game.in_a_pointdevice_run();
        game.one_frame();
        let log = game.log();

        assert!(
            log.said("lives: no panel tile; the strips are painted flat and will show as a patch"),
            "orb did not say the panel was being painted flat:\n  {}",
            log.lines().join("\n  ")
        );
        let (texture, _) = game.image().the_panels_tile();
        assert!(
            game.drawn()
                .quads
                .iter()
                .all(|quad| quad.texture != texture),
            "a tile orb could not find was bound to paint the panel with",
        );
        // And the mark is still there: a strip painted flat is a patch, and no mark at all is a count that
        // reads as lives somebody has.
        assert_eq!(game.says("DISABLE").len(), 1, "the mark went with the tile");
    });
}
