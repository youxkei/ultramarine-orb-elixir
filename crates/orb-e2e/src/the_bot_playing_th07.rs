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
//! - the shot held, let go for a frame every second so that a screen waiting for a press gets one, and Ctrl
//!   while a dialogue is up
//! - the bomb the frame the player is hit, which is 妖々夢's own 決死結界 — [`PLAYER_STATE`] stands at 2 while
//!   the game counts a window down, and a bomb inside it is a death taken back
//! - **the nearest to where it wants to be, of the nine moves nothing will be at**: holding each is followed
//!   three frames out against every bullet's own step, so a bullet is stepped around rather than fled from and
//!   the item behind it is still gone for. An enemy counts as much as a bullet, its body being as solid here
//! - and with nothing near, whichever item is worth most — a life, a bomb, the big power item, the small one,
//!   the point item, the cherry — or the boss's own x with the y low, or the middle of the field low
//! - **all of it inside the bottom third of the field**, dodges included, which is what three launches that
//!   went up and stayed up bought
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
/// How often the shot is let go for a frame, as `bot::SHOT_LET_GO_EVERY` has it.
const SHOT_LET_GO_EVERY: u32 = 60;

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

/// **And lets it go for a frame now and then, so that a screen waiting for a press gets one.** The stage's own
/// clear screen goes on to the next stage the frame the shot goes down and otherwise waits out its own timer,
/// and a run holding the shot since the stage began never gives it that frame: one launch spent 60 seconds
/// between 1面's last chapter and 2面's first.
///
/// Which is a rule about the *edge* and not about the fire rate. A frame in every
/// [`the_shot_is_let_go_every`](Self) is a shot in sixty, and 妖々夢's own shot fires off the player's timer
/// rather than off a press.
#[test]
fn the_bot_lets_the_shot_go_for_a_frame_now_and_then() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-lets-the-shot-go");
        let mut let_go = 0;
        let mut runs = 0;
        let mut holding = true;
        for _ in 0..SHOT_LET_GO_EVERY * 2 {
            game.frame();
            let held = game.image().the_word_the_run_acts_on() & SHOT != 0;
            if !held {
                let_go += 1;
                if holding {
                    runs += 1;
                }
            }
            holding = held;
        }

        assert_eq!(
            (let_go, runs),
            (2, 2),
            "the shot was not let go for one frame twice over {} frames:\n  {}",
            SHOT_LET_GO_EVERY * 2,
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

/// The kinds of item these place, as the table at 0x433a5a indexes them: 0 steps the power up by one and 2 by
/// eight — 大パワーアイテム — 5 is the life and 3 the bomb, 1 is worth a score off the chain and 7 fills the
/// cherry. 9 is more cherry, and stands here for a kind the rules give no place of its own.
const POWER: i8 = 0;
const POINT: i8 = 1;
const BIG_POWER: i8 = 2;
const BOMB_ITEM: i8 = 3;
const LIFE: i8 = 5;
const CHERRY: i8 = 7;
const NO_PLACE: i8 = 9;

/// A field for these to place things in, as the game's own clamp reads it: left, top, width, height.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
/// The y the rules wait at in that field, off its bottom edge — and the x they wait on with nothing to be
/// under, which is the middle of it.
const LOW: f32 = 400.0;
const MIDDLE: f32 = 192.0;
/// And the top of the band it moves in, which is the bottom third of that field.
const BAND_TOP: f32 = FIELD.1 + FIELD.3 * 2.0 / 3.0;
/// What one frame of holding a direction moves the player by in these, straight.
const A_STEP: f32 = 4.0;

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
    // A step of the shape the game's own is: a diagonal covers a corner of the square a straight move covers a
    // side of, so the two are not the same distance and a diagonal is not simply the better move.
    game.image()
        .the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
    // And a box for it to be hit in. With a bullet of no size of its own this is the whole of what a hit is, so
    // these read as "nearer than a step on both axes and it is a hit".
    game.image().the_players_box_is((A_STEP, A_STEP));
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

/// **The order the items are taken in**, which is what each is worth: a life, then a bomb, then the big power
/// item over the small one, then the point item, then the cherry — and last, any kind these rules give no place
/// of its own, so that nothing on the screen is walked past.
///
/// Each in turn is taken away so that which way the bot steers says which of what is left it went for. All of
/// them level with the player, so the answer is left or right and nothing else.
#[test]
fn the_bot_takes_the_items_in_their_own_order() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-them-in-order");
        game.image().the_player_is_at(MIDDLE, LOW);
        let (left, right) = (40.0, 340.0);
        // The slot, the kind and the side it is on, in the order they are worth going for.
        let order = [
            (1, LIFE, left, LEFT),
            (2, BOMB_ITEM, right, RIGHT),
            (3, BIG_POWER, left, LEFT),
            (700, POWER, right, RIGHT),
            (800, POINT, left, LEFT),
            (900, CHERRY, right, RIGHT),
            (1000, NO_PLACE, left, LEFT),
        ];
        for (slot, kind, x, _) in order {
            game.image().an_item_is_at(slot, kind, x, LOW);
        }

        for (slot, kind, _, wanted) in order {
            game.frames(1);
            assert_eq!(
                game.image().the_word_the_run_acts_on() & PAD,
                wanted,
                "the bot did not go for kind {kind} while it was the best on the screen:\n  {}",
                game.log().lines().join("\n  "),
            );
            game.image().the_item_is_gone(slot);
        }

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot steered with the screen empty and itself where it waits:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The bottom third of the field is the whole of where it moves**, whatever is above it: an item, a boss, or
/// the collection at the top of the field that every item comes in to.
///
/// Which is a bound and not a preference — three launches went up for one of those and stayed up, and each time
/// the reason it stayed was a different one. What the band gives up is the collection at the top; what it buys
/// is a run that is never up among the bullets for anything.
#[test]
fn the_bot_stays_in_the_bottom_third_of_the_field() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-stays-in-its-band");
        game.image().the_player_is_at(MIDDLE, LOW);
        // As high up the field as an item goes, so that nothing but the band decides what is done about it.
        game.image().an_item_is_at(700, BIG_POWER, MIDDLE, 8.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot came up out of its band for an item above it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And one inside the band is gone up to, that being a place it may be.
        game.image().the_item_is_gone(700);
        game.image()
            .an_item_is_at(800, BIG_POWER, MIDDLE, BAND_TOP + 8.0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            UP,
            "the bot did not come up for an item inside its own band:\n  {}",
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
            // Across and not up-and-across: the big one is above the band, so what is gone to is its x at the
            // line the bot waits on — see [`the_bot_stays_in_the_bottom_third_of_the_field`].
            LEFT,
            "the bot took the small power item under it over the big one:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy is dodged as much as a bullet**, its body being as solid as one in 妖々夢 — and it is what a run
/// on its way to an item walks into, the item having been dropped by whatever is still standing over it.
#[test]
fn the_bot_dodges_an_enemy_on_its_way_to_an_item() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-dodges-an-enemy");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The item to the left, and the enemy that dropped it between the two.
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        game.image().an_enemy_is_at(9, MIDDLE - 8.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT | UP,
            "the bot walked into the enemy on its way to the item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A move is followed by the step the game would really take**, and not by a length of the rules' own
/// choosing.
///
/// Which is what a boss fight cost: a bot that stepped aside by twice what it really moved left itself in the
/// bullet's path every time, and every hit it took was a bomb — bombs 3, 2, 1, 0 through 1面's boss with the
/// deaths still at zero. So the step is read where the game works it out, and here it is read back: with an item
/// half a long step away the bot stays where it is, a step being an overshoot, and with a short one it moves.
#[test]
fn the_bot_follows_a_move_by_the_games_own_step() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-steps-as-the-game-does");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, MIDDLE + 5.0, LOW);

        game.image().the_player_steps_by(10.0, 7.0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stepped ten to close five:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_player_steps_by(2.0, 1.4);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot stood still with a step short enough to close on the item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet is stepped around and not fled from.** The next frames' positions are known, so what a move has
/// to be is somewhere none of them will be — keeping *away* from bullets besides is a bot that spends the stage
/// leaving, and the item it was on its way to is still worth going to behind one that has passed.
#[test]
fn the_bot_steps_around_a_bullet_rather_than_fleeing_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-steps-around");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        // Coming down onto where the player stands: a step to the side is out of its way, and the item is that
        // way too.
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - 24.0, (0.0, 8.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot fled the bullet instead of stepping out from under it to the item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet's own size is part of what a hit is**, and no two kinds of bullet have the same one.
///
/// The game's test is two boxes overlapping — the player's own and the bullet's place give or take half its
/// size — so a number standing in for every bullet is a different test rather than an approximation of that one.
/// Which two launches paid for: the hits they reported had 2.1 and 3.7 units of room by that number's reckoning.
///
/// Here the same place is safe for a bullet of no size and a hit for a wide one.
#[test]
fn a_bullets_own_size_is_part_of_what_a_hit_is() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-bullets-own-size");
        game.image().the_player_is_at(MIDDLE, LOW);
        // A step and a half out, which the player's own box alone does not reach.
        let out = MIDDLE + A_STEP * 1.5;

        game.image().a_bullet_is_at(500, out, LOW, (0.0, 0.0));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for a bullet of no size that could not reach it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same place, with a bullet wide enough that its own half closes the gap.
        game.image().a_bullet_of_size_is_at(
            500,
            out,
            LOW,
            (0.0, 0.0),
            (A_STEP * 2.0, A_STEP * 2.0),
        );
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood inside a bullet wide enough to reach it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where nothing is open, the move that stays clear longest is taken.**
///
/// Which is what a pattern with no way out of it needs: the most *room* is not the same as the most time, and
/// a launch that took the roomiest move walked into the next frame's wall with it. Three frames of clear is
/// three frames for the pattern to move on in.
#[test]
fn the_bot_takes_the_longest_way_out_where_nothing_is_open() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-lasts-longest");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Sitting on where it stands and on every place one step of it leads, so that the corners are shut by
        // those too — and one more out to the right, which the third step of going that way reaches and the
        // first two do not.
        for (slot, (x, y)) in [
            (MIDDLE, LOW),
            (MIDDLE, LOW - A_STEP),
            (MIDDLE, LOW + A_STEP),
            (MIDDLE - A_STEP, LOW),
            (MIDDLE + A_STEP * 3.0, LOW),
        ]
        .into_iter()
        .enumerate()
        {
            game.image().a_bullet_is_at(slot + 1, x, y, (0.0, 0.0));
        }

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot did not take the one way that lasted:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A hit says what the rules thought of the move that took it.**
///
/// Which is the whole of what is left to find out. A frame ahead is computed and a move nothing will be at is
/// chosen, so a run that is hit at all means one of three things and no others: something on the screen is not
/// in the model — the lasers are not, and a bullet that turns may not move the way one step of its own says —
/// or every one of the nine was shut, or the move chosen was not the move made. The line says which: the room
/// the model left, or that it had none to leave.
#[test]
fn a_hit_says_what_the_rules_thought_of_the_move_that_took_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-says-what-was-thought");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert!(
            !game.log().said("autoplay: hit"),
            "a hit was said on a frame nothing had hit the player on",
        );

        game.image().the_player_has_been_hit();
        game.frames(1);
        assert!(
            game.log().said("autoplay: hit while"),
            "the hit did not say what the rules thought of the move:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step aside is a way out and not another hit.** Which needs the room a move opens to be wider than what
/// counts as being hit: where the two are the same, every neighbour of a place a bullet crosses is a hit too,
/// so no move is open, and the choice falls through to the most room — which is the fleeing the whole of this
/// exists instead of, and fleeing a boss's pattern is 決死結界 after 決死結界.
///
/// A bullet coming straight down the player's own column, arriving where it stands and with a step's room on
/// every side: one of those steps has to be taken, and taken straight — a corner is what the most room picks
/// when nothing is open, so a corner here says nothing was.
#[test]
fn the_bot_steps_out_of_the_way_of_a_bullet_a_step_wide_of_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-has-room-to-step");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - A_STEP, (0.0, A_STEP));

        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            [UP, DOWN, LEFT, RIGHT].contains(&pressed),
            "the bot did not take one of the steps open to it, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And **the frames after the next one count too**: a bullet three frames from where the player stands is a
/// bullet to be somewhere else for, and one frame of warning is not enough to be anywhere.
#[test]
fn the_bot_leaves_for_a_bullet_that_is_frames_away() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-looks-ahead");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Three of its own steps above the player and coming down onto it: a frame ahead it is still clear.
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - 24.0, (0.0, 8.0));

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood where a bullet would be in three frames:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a bullet it has room from stops nothing.** A band wide enough to take in most of what is on the screen
/// is a bot that spends every frame leaving and never collects anything: the screen of a 妖々夢 stage is never
/// empty, and being somewhere near a bullet is the ordinary state of playing at all.
#[test]
fn the_bot_is_not_stopped_by_a_bullet_it_has_room_from() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-keeps-going");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        // Four steps up the field, which is room enough that this frame is not about it.
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - 32.0, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot stopped collecting for a bullet it had room from:\n  {}",
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

        // **And anything that could reach it takes the slowing off again, hit or not**: a dodge at half a step
        // is half a dodge, and by the frame a bullet is a hit the halving has already been paid for. This one is
        // three of its own steps away and going nowhere near — a hit next frame it is not, and it is close
        // enough to want the whole step back.
        game.image()
            .a_bullet_is_at(500, MIDDLE + A_STEP * 3.0, LOW, (0.0, 0.0));
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

/// A slot whose item has been collected is not an item, whatever is still written in it.
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
            // Across and not up-and-across: the item is most of a field away to the side and a little way up,
            // so the step that closes the most of that is the straight one. The climb comes once the x is met.
            RIGHT,
            "the bot stood under the boss with power on the screen:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
