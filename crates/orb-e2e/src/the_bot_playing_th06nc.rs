//! **The rules over th06nc's own cards**, laid out: what they read of a card and where its way puts them —
//! `the_bot_playing_th06c.rs` for the ways that game's Extra was cleared by, over this game's numbers.
//!
//! **What a card's way is handed of how far into the card it is, is the boss's attack timer** —
//! `enemy::TIMER` in `th06nc/mod.rs`, which `runtime` hands on as the card's frames while a card is up. A
//! launch of the real game that read no timer had every card at frame nought, and `「そして誰もいなくなるか？」`'s
//! way never reached its second half: it led the chasers round the corners to the card's end and stood in the
//! bottom right, where it waits at the bottom middle.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// `秘弾「そして誰もいなくなるか？」` by this game's own number for it — th06c's 62.
const AND_THEN_THERE_WERE_NONE: u32 = 129;

/// A frame of the card past its chasers' coming out of the corners, which is where its second half is.
const IN_THE_SECOND_HALF: i32 = 2000;

/// **Where the rules want to go from `at`**, off the line the card's own dodge says it with.
fn wanted_from(game: &Fake, at: (f32, f32)) -> Option<(f32, f32)> {
    game.image().the_player_is_at(at.0, at.1);
    game.frames(2);
    game.log().lines().iter().rev().find_map(|line| {
        let (_, after) = line.split_once("wanted the place the card's own dodge puts it at ")?;
        let (place, _) = after.split_once(" from ")?;
        let (x, y) = place.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    })
}

/// **Once its chasers are all gone the rules wait the card out at the bottom middle**, a quarter of the field
/// up from its floor — the laid-out field being `448` high, `336` — from the bottom right they were left in.
#[test]
fn the_rules_of_th06nc_wait_the_card_out_low_in_the_middle_under_and_then_there_were_none() {
    in_its_own_process(|| {
        const THE_BOTTOM_MIDDLE_A_QUARTER_UP: (f32, f32) = (192.0, 336.0);
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-and-then-there-were-none-waited-out",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: and_then_there_were_none, spell: {AND_THEN_THERE_WERE_NONE} }}\n\
                     - play_until: {{ frames: 4000 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.in_a_run_in_the_stage(6);
        game.fires_at_the_player(false);
        game.image().a_boss_is_fighting(1000, IN_THE_SECOND_HALF);
        game.image().the_spellcard_is(AND_THEN_THERE_WERE_NONE);
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        let wanted = wanted_from(&game, (336.0, 400.0));
        assert!(
            wanted.is_some_and(
                |place| (place.0 - THE_BOTTOM_MIDDLE_A_QUARTER_UP.0).abs() < 1.0
                    && (place.1 - THE_BOTTOM_MIDDLE_A_QUARTER_UP.1).abs() < 1.0
            ),
            "put in the bottom right with no chaser left the rules wanted {wanted:?} and not the bottom \
             middle {THE_BOTTOM_MIDDLE_A_QUARTER_UP:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
