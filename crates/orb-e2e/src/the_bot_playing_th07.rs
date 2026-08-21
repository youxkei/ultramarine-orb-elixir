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
//! The rules are 妖々夢 on Normal, and dodging comes before all of them:
//!
//! - the shot held, always, and Ctrl while a dialogue is up
//! - the bomb the frame the player is hit, which is 妖々夢's own 決死結界 — [`PLAYER_STATE`] stands at 2 while
//!   the game counts a window down, and a bomb inside it is a death taken back
//! - **a bullet near the player and the frame is spent leaving**: the safest of the nine one-frame moves,
//!   which is why up and down are dodges as much as left and right
//! - and with nothing near, the way to a power item — the big one over a small one wherever both are up — or
//!   the boss's own x with the y low, or the middle of the field low
//! - none of it pressed off a stage, under the pause menu, or on an update with no stage built yet
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
/// The one that skips a dialogue, which is Ctrl, and the one that slows the player down, which is Shift.
const SKIP: u16 = 0x100;
const FOCUS: u16 = 0x004;

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
        let game = in_a_stage("th07-the-bot-shoots");

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
        let game = in_a_stage("th07-the-bot-death-bombs");
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

/// The kinds of item these place, as the table at 0x433a5a indexes them: 0 steps the power up by one, 2 by
/// eight — 大パワーアイテム — and 1 is the one worth a score off the chain.
const POWER: i8 = 0;
const POINT: i8 = 1;
const BIG_POWER: i8 = 2;

/// A field for these to place things in, as the game's own clamp reads it: left, top, width, height.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
/// The y the rules hold in that field, off its bottom edge — and the x they wait on with nothing to be under,
/// which is the middle of it.
const LOW: f32 = 400.0;
const MIDDLE: f32 = 192.0;

/// A stage running, with a field to move about in and nobody fighting — what the rules about where to be are
/// read against.
///
/// The supervisor's own scene set to a run's and no pause menu, because those two are what the rules read to
/// know a stage is running at all — see [`the_bot_presses_nothing_at_the_front_end`].
fn in_a_stage(name: &str) -> Box<Fake> {
    let game = playing(name);
    game.image().the_player_is_playing();
    game.image().is_playing(1, 1);
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

/// A power item is gone to, which is what a run of 妖々夢 spends a stage's way through doing — and gone to
/// where it is, up the field as much as across it.
#[test]
fn the_bot_goes_for_a_power_item() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-power");
        game.image().the_player_is_at(100.0, LOW);
        // A slot well up the manager's array, because which slot an item is in is the whole question its walk
        // answers — a rule that only ever looked at the first would pass a laid-out game and miss every launch.
        game.image().an_item_is_at(700, POWER, 200.0, 300.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT | UP,
            "the bot did not go up and across to the power item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And **the big one over a small one wherever both are on the screen**, whatever the distances: 妖々夢's
/// 大パワーアイテム is eight steps of power against the small one's one, so a stage that drops one drops the
/// thing worth crossing the screen for.
#[test]
fn the_bot_goes_for_a_big_power_item_over_a_small_one() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-the-big-power");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The small one under the player's own hand and the big one across the field, so that distance alone
        // would pick the wrong one.
        game.image().an_item_is_at(11, POWER, MIDDLE, LOW);
        game.image().an_item_is_at(900, BIG_POWER, 40.0, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT | UP,
            "the bot took the small power item under it over the big one:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet near the player is what the frame is spent on, whatever else is on the screen.** Every bomb this
/// bot spends is a hit it could have moved out of, so dodging comes before the power it was on its way to.
#[test]
fn the_bot_dodges_before_it_collects() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-dodges");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The power item to the left, and a bullet on the way to it.
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        game.image()
            .a_bullet_is_at(500, MIDDLE - 8.0, LOW, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot walked into the bullet on its way to the item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And **up and down are dodges as much as left and right**: hemmed in on both sides, the way out is the axis
/// nothing is on.
#[test]
fn the_bot_dodges_upwards_where_both_sides_are_shut() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-dodges-upwards");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(3, MIDDLE - 16.0, LOW, (0.0, 0.0));
        game.image()
            .a_bullet_is_at(900, MIDDLE + 16.0, LOW, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            UP,
            "the bot stood between two bullets rather than leaving:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// A bullet is dodged by where it **will be**, not where it is: its own update adds its step to its position
/// before the frame this word is played on.
///
/// Down and to the left of the player and going down and to the right, which is the case that tells the two
/// apart — away from where it *is* is to the right, and away from where it is *going* is up.
#[test]
fn the_bot_dodges_where_the_bullet_is_going() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-dodges-ahead");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(500, MIDDLE - 16.0, LOW, (16.0, 16.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            UP,
            "the bot dodged where the bullet was rather than where it was going:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A boss is shot at 低速移動**, which is where 妖々夢's shots do their damage — and only where there is
/// nothing to dodge, the same key halving the step a dodge is made of.
#[test]
fn the_bot_shoots_a_boss_slowed_down() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-focuses-on-a-boss");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            0,
            "the bot slowed down with no boss to shoot at",
        );

        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(MIDDLE, 100.0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            FOCUS,
            "the bot shot the boss at full speed:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And a bullet near it takes the slowing off again: a dodge at half a step is half a dodge.
        game.image()
            .a_bullet_is_at(500, MIDDLE - 8.0, LOW, (0.0, 0.0));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            0,
            "the bot dodged at half a step:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// A dialogue is skipped, which is 妖々夢's own Ctrl: a bot that sat through every one of them would spend the
/// afternoon it was written to save.
#[test]
fn the_bot_skips_a_dialogue() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-skips-a-dialogue");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & SKIP,
            0,
            "the bot skipped with no dialogue up",
        );

        game.image().has_a_dialogue();
        game.image().the_dialogue_is_at_line(0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & SKIP,
            SKIP,
            "the bot did not skip the dialogue:\n  {}",
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

/// **Nothing at all is pressed while a stage is still being built**, the shot included: the player and the
/// play field read as zeroes on those updates, so a rule that pressed there would be pressing on numbers that
/// are not the run's yet — which is the very stretch a resume is measured across.
#[test]
fn the_bot_presses_nothing_while_a_stage_is_being_built() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-mid-build");
        game.image().the_stage_is_being_rebuilt();
        game.image().the_play_field_is(0.0, 0.0, 0.0, 0.0);
        game.image().the_player_is_at(0.0, 0.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            0,
            "the bot pressed on an update with no stage built yet:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a run's numbers being there is not a stage running.** The front end is up with the pointer to them
/// still set — which is the launch that walked 妖々夢's own title menu onto 終了 and took it: nothing had been
/// played, so the play field read as zeroes, so the y the rules wanted was above the field's own top, so
/// [`UP`] went in and the cursor wrapped from the first item to the last.
#[test]
fn the_bot_presses_nothing_at_the_front_end() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-at-the-front-end");
        game.image().the_front_end_is_up();
        // Everything a stage leaves behind still where it was, and the field never filled in: the state the
        // launch was really in, rather than one tidied up first.
        game.image().the_play_field_is(0.0, 0.0, 0.0, 0.0);
        game.image().the_player_is_at(0.0, 0.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            0,
            "the bot pressed at the front end:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// Nor while the run is paused, which is 妖々夢's own pause menu — the same shot, and the same 十字キー.
#[test]
fn the_bot_presses_nothing_while_the_run_is_paused() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-paused");
        game.image().the_player_is_at(FIELD.0, FIELD.1);
        game.image().is_paused();

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            0,
            "the bot walked the pause menu:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A power item is gone for during a fight too.** A boss drops them, and power is what shortens the next
/// fight — the boss's own x is where to be with nothing better to do, not a place to be held to.
#[test]
fn a_power_item_takes_the_bot_off_a_boss() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-collects-during-a-fight");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(100.0, 100.0);
        game.image().the_player_is_at(100.0, LOW);
        game.image().an_item_is_at(700, POWER, 380.0, 20.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT | UP,
            "the bot stood under the boss with power on the screen:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
