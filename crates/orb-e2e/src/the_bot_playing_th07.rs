//! **orb playing 妖々夢 itself, so that a run to be picked up out of a file costs nobody an afternoon.**
//!
//! Every measurement of a resume so far has cost somebody a run of stage 1 and a stretch of stage 2, and then
//! another one every time a field was added to what the file carries. A run the machine plays is a run that can
//! be had again on demand — and the same buttons every launch, which is what makes two launches comparable at
//! all.
//!
//! **It plays the way somebody does and reads what they read.** No invulnerability and no keys sent from
//! outside: the word goes in where a playback's own words go in — see `resume::fed` and the read hook in
//! `runtime` — and what it is made of is the game's own numbers, read where the game keeps them.
//!
//! The rules are 妖々夢 on Normal, where the bullets are thin enough that a run does not need to dodge:
//!
//! - the shot held, always
//! - the bomb the frame the player is hit, which is 妖々夢's own 決死結界 — [`PLAYER_STATE`] is 2 for the
//!   updates between the hit test finding a bullet and the death handler running, so a bomb pressed there is a
//!   death taken back
//! - the y held low in the play field, always — a power item falls, so lining its x up and waiting under it
//!   is being handed the item without climbing into the bullets for it
//! - the x held on the boss under a boss, on the nearest power item where one is on the screen, and in the
//!   middle of the field with neither
//! - and none of the four steered off a stage or under the pause menu, where the same 十字キー walks a menu
//!
//! What no laid-out game can say is whether the rules clear the game. That is what a launch says, and the
//! reason this file holds the rules one at a time instead: a run that dies in stage 3 is a rule that needs
//! another look, and which rule it was is what these say.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// 妖々夢's own first two buttons, as `Th07` has them: shoot and bomb.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;

fn playing(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            // A recipe of no steps, which is a run played from its first frame: what these two are about is
            // the rules and not the screens on the way in.
            config.autoplay = Some("[]".to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.streams_its_song(0);
    // **Nothing of the menus driven here.** A launch playing to a recipe answers every read of the game's own
    // keyboard, so a test that pressed its way into a run would be pressing against the bot — and the word
    // each frame was handed is written where a test can read it whatever screen is up. What these two are
    // about is the word, and the player's own state is put where they want it outright.
    game
}

/// The shot is held on every frame of a stage, which is the whole of what a run of 妖々夢 does with it.
#[test]
fn the_bot_holds_the_shot_down() {
    in_its_own_process(|| {
        let game = playing("th07-the-bot-shoots");
        game.image().the_player_is_playing();

        game.frames(30);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & SHOT,
            SHOT,
            "the bot played a frame of a stage without the shot held:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And the bomb on the frame the player is hit, which is what takes the death back.
///
/// **The state and not the deaths**, because a death counted is a death already taken: [`PLAYER_STATE`] is 2
/// from the update the hit test finds a bullet on until the one the death handler runs on, and those are the
/// updates a bomb is still a 決死結界.
#[test]
fn the_bot_bombs_the_frame_the_player_is_hit() {
    in_its_own_process(|| {
        let game = playing("th07-the-bot-death-bombs");
        game.image().the_player_is_playing();
        game.frames(30);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            0,
            "the bot bombed a frame nothing had hit the player on",
        );

        game.image().the_player_has_been_hit();
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            BOMB,
            "the bot did not bomb the frame the player was hit:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// 妖々夢's own 十字キー bits, as `Th07` has them.
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
/// All four, for an assertion about a frame nothing should have been steered on.
const PAD: u16 = UP | DOWN | LEFT | RIGHT;

/// The two kinds of item these place, as the table at 0x433a5a indexes them: 0 is the one that steps the power
/// up and 1 is the one worth a score off the chain.
const POWER: i8 = 0;
const POINT: i8 = 1;

/// A field for these to place things in, as the game's own clamp reads it: left, top, width, height.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
/// The y the rules hold in that field, off its bottom edge — and the x they wait on with nothing to be under,
/// which is the middle of it.
const LOW: f32 = 400.0;
const MIDDLE: f32 = 192.0;

/// A stage running, with a field to move about in and nobody fighting — what the rules about where to be are
/// read against.
///
/// A run's own numbers and no pause menu, because those two are what the rules read to know a stage is running
/// at all: a screen with neither is a screen 十字キー walks a menu on.
fn in_a_stage(name: &str) -> Box<Fake> {
    let game = playing(name);
    game.image().the_player_is_playing();
    game.image().has_a_runs_numbers();
    game.image().is_not_paused();
    game.image()
        .the_play_field_is(FIELD.0, FIELD.1, FIELD.2, FIELD.3);
    game.image().no_boss_is_fighting();
    game
}

/// The player is held low whatever else is going on, which is where a power item falling reaches it and where
/// a fight is survivable.
#[test]
fn the_bot_keeps_the_player_low() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-stays-low");
        game.image().the_player_is_at(MIDDLE, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            DOWN,
            "the bot left the player high up the field:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_player_is_at(MIDDLE, FIELD.3);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            UP,
            "the bot left the player against the bottom edge:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// A power item's x is lined up with, which is what a run of 妖々夢 spends a stage's way through doing: the
/// item falls, so the x is the whole of what has to be got right.
#[test]
fn the_bot_lines_up_with_a_power_item() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-power");
        game.image().the_player_is_at(100.0, LOW);
        // A slot well up the manager's array, because which slot an item is in is the whole question its walk
        // answers — a rule that only ever looked at the first would pass a laid-out game and miss every launch.
        game.image().an_item_is_at(700, POWER, 200.0, 300.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot did not line up with the power item to its right:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And with nothing on the screen to be under, the middle of the field — the shortest way to wherever the next
/// item falls.
#[test]
fn the_bot_waits_in_the_middle_of_the_field() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-waits-in-the-middle");
        game.image().the_player_is_at(FIELD.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot waited against the field's left edge:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And the nearest of them, where there is more than one.
#[test]
fn the_bot_goes_for_the_nearest_power_item() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-the-nearest-power");
        game.image().the_player_is_at(200.0, LOW);
        game.image().an_item_is_at(3, POWER, 100.0, LOW);
        game.image().an_item_is_at(900, POWER, 380.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot went for the far power item over the near one:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// An item that is not a power item is left where it is. The rules are about power, and what 妖々夢 does with
/// the rest of them is collect the screen the moment the border goes up.
#[test]
fn the_bot_leaves_an_item_that_is_not_power() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-point-item");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POINT, 380.0, 300.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot steered towards an item that was not a power item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And a slot whose item has been collected is not an item, whatever is still written in it.
#[test]
fn the_bot_leaves_a_slot_whose_item_is_gone() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-collected-item");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, 380.0, 300.0);
        game.image().the_item_is_gone(700);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot steered towards a slot whose item had been collected:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// Under a boss the player holds the boss's own x and stays low, which is a fight survivable without reading
/// the bullets.
#[test]
fn the_bot_holds_the_bosss_x_and_stays_low() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-fights-a-boss");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(300.0, 100.0);
        game.image().the_player_is_at(100.0, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT | DOWN,
            "the bot did not steer under the boss, to its right and below it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And presses nothing once it is there, rather than stepping across the place it wants to be and back every
/// frame.
#[test]
fn the_bot_stops_where_it_wanted_to_be() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-stops-under-a-boss");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(300.0, 100.0);
        game.image().the_player_is_at(300.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot steered away from the place it had reached:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// Nothing is steered where no stage is running. A run that died out of a `play_until` is a launch sitting on
/// the title screen with the rules still being asked for a word, and 十字キー there is a menu being walked
/// through by something that cannot read it.
#[test]
fn the_bot_steers_nothing_off_a_stage() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-off-a-stage");
        game.image().the_player_is_at(FIELD.0, FIELD.1);
        game.image().has_no_runs_numbers();

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot steered with no run to steer in:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// Nor while the run is paused, where the same 十字キー walks 妖々夢's own pause menu.
#[test]
fn the_bot_steers_nothing_while_the_run_is_paused() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-paused");
        game.image().the_player_is_at(FIELD.0, FIELD.1);
        game.image().is_paused();

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot walked the pause menu:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// A boss takes the bot off the items, an item across the screen during a fight being a fight left to walk
/// back into.
#[test]
fn a_boss_takes_the_bot_off_the_items() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-fights-rather-than-collects");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(100.0, 100.0);
        game.image().the_player_is_at(100.0, LOW);
        game.image().an_item_is_at(700, POWER, 380.0, 20.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left the boss to go for an item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
