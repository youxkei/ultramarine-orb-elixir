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
/// **The widest bullet 妖々夢 makes**, which is 大玉: the spawn writes each kind's own size into `+0xb7c` and
/// `+0xb80`, and of the sizes it writes — 4, 5, 6, 8, 10 — this is the one at 0x4275ca.
const WIDEST_BULLET: f32 = 24.0;

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
        // The item to the left, and the enemy that dropped it between the two — far enough out that going round
        // it is a move with [`ROOM_TO_SPARE`] to spare, which is what makes going round it possible at all.
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        game.image().an_enemy_is_at(9, MIDDLE - A_STEP * 3.0, LOW);

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

/// **What the bot presses is written down**, or the whole harness is pointless.
///
/// A run orb kept no buttons for is a run nothing can pick up, and a run to pick up is the one thing the bot
/// exists to make: the first launch that played its way to 2面 left `resume: not written — 0 frame(s) of buttons`
/// behind it, because the word went to the game by a path of its own instead of through the one place a read is
/// noted.
#[test]
fn what_the_bot_presses_is_written_down_for_the_run() {
    in_its_own_process(|| {
        let game = playing("th07-the-bot-is-written-down");
        game.in_a_pointdevice_run();

        game.frames(120);
        assert!(
            !game.log().said("0 frame(s) of buttons"),
            "the run kept none of what the bot pressed:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A laser that turns is left by how much it turned**, which is a thing to remember rather than to read.
///
/// Nothing in a laser's own update moves its angle — 0x424f6e writes it where one is made and its boss's script
/// moves it after that — so there is no field to project from. What there is is the angle it had last frame, and
/// the difference is the turn.
///
/// A beam through the top of the field, missing the player by a good margin and swinging towards it: still clear
/// where it points now, and over the frames looked at it arrives.
#[test]
fn the_bot_leaves_a_laser_that_is_turning_onto_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-turning-laser");
        game.image().the_player_is_at(MIDDLE, LOW);
        let straight_down = std::f32::consts::FRAC_PI_2;
        let beam = |off: f32| {
            game.image().a_laser_is_at(
                11,
                (MIDDLE, FIELD.1),
                straight_down - off,
                (0.0, FIELD.3),
                A_STEP * 2.0,
            );
        };

        // The first frame has nothing to compare with, so the beam is only where it points — wide of the player.
        beam(0.05);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left a beam that was pointing wide of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And now it has turned a little towards it. Where it points is still wide; where it is going is not.
        beam(0.04);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood under a beam that was swinging onto it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy's own body is part of what a hit is**, a boss being a wide thing to stand under.
///
/// Taken for a point, the model says clear where the body reaches — which is what a hit with room to spare looks
/// like, and the run that reported `with 4.4 of room` was on a boss. The game hands its own test the size at
/// `+0x2b3c` and a third of it on each side is what arrives.
#[test]
fn an_enemys_own_body_is_part_of_what_a_hit_is() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-an-enemys-own-body");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Two steps out, which the player's own box alone does not reach.
        let out = MIDDLE + A_STEP * 2.0;

        game.image().an_enemy_is_at(4, out, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for a body that could not reach it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same place with a body wide enough that a third of it closes the gap.
        game.image()
            .an_enemy_of_size_is_at(4, out, LOW, (A_STEP * 6.0, A_STEP * 6.0));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood inside a body wide enough to reach it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A laser is left, which the bullets and the enemies alone never were.**
///
/// A launch that could not see them was hit eighteen times on 4面's boss, and every one of those hits said the
/// move it was taken on had twenty units of room — which is not a hitbox got wrong but a hazard that is not in
/// the model at all.
///
/// One straight down the player's own column and wide enough to cover it, so the way out is sideways; and then
/// the same one past the phase the game's own walks stop looking at, which is nothing to leave for.
#[test]
fn the_bot_leaves_a_laser_that_is_firing() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-laser");
        game.image().the_player_is_at(MIDDLE, LOW);
        // From the top of the field straight down, reaching past where the player stands, and a step wide.
        game.image().a_laser_is_at(
            7,
            (MIDDLE, FIELD.1),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            A_STEP * 2.0,
        );

        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed == LEFT || pressed == RIGHT,
            "the bot stood in a firing laser, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_laser_is_fading(7);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left for a laser the game had stopped looking at:\n  {}",
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
        // **A step long beside the box and beside [`ROOM_TO_SPARE`]**, so that one of the nine landings can be
        // shut without shutting the ones either side of it: with the ordinary step every landing is inside its
        // neighbour's room to spare, and a pattern that shuts one shuts them all.
        let (step, across) = (A_STEP * 4.0, A_STEP * 4.0 / 2.0_f32.sqrt());
        game.image().the_player_steps_by(step, across);
        game.image().the_players_box_is((1.0, 1.0));
        // Sitting on every place one step leads, the corners included — and one more out to the right, which
        // the third step of going that way reaches and the first two do not.
        for (slot, (x, y)) in [
            (MIDDLE, LOW),
            (MIDDLE, LOW - step),
            (MIDDLE, LOW + step),
            (MIDDLE - step, LOW),
            (MIDDLE - across, LOW - across),
            (MIDDLE + across, LOW - across),
            (MIDDLE - across, LOW + across),
            (MIDDLE + across, LOW + across),
            (MIDDLE + step * 3.0, LOW),
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

/// **A hit says what the rules thought of the move that took it — and that is the frame before's move.**
///
/// Which is the whole of what is left to find out. A frame ahead is computed and a move nothing will be at is
/// chosen, so a run that is hit at all means one of three things and no others: something on the screen is not
/// in the model, or every one of the nine was shut, or the move chosen was not the move made. The line says
/// which: the room the model left, or that it had none to leave.
///
/// **The frame the state says hit is a frame the hit has already been taken off**: 0x425eb7 puts the bullet's
/// own state to 5 the moment the test comes back, so this frame's judgement is of a screen without the thing
/// that did it, and its room is the room to something else. So the move reported is the one played on the frame
/// the hit came out of, which is what this asks: an item to the right pulls the bot right, and the line said
/// after the item has gone and the player is hit is still that right.
#[test]
fn a_hit_says_what_the_rules_thought_of_the_move_that_took_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-says-what-was-thought");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, MIDDLE + 100.0, LOW);
        game.frames(1);
        assert!(
            !game.log().said("autoplay: hit"),
            "a hit was said on a frame nothing had hit the player on",
        );
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot did not go for the item to its right:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_item_is_gone(700);
        game.image().the_player_has_been_hit();
        game.frames(1);
        assert!(
            game.log().said("having pressed 0x080"),
            "the hit did not say the move the frame it came out of was played with:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And where the player was on it, which is the other half of reading a hit: a room that is nearly
        // enough says the model missed by a little, and only the place says whether it missed at a wall.
        assert!(
            game.log().said(&format!("hit at {MIDDLE:.1},{LOW:.1}")),
            "the hit did not say where the player was:\n  {}",
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

        // **A bullet near the player but going nowhere leaves the slowing on.** Being near is not being a
        // threat, and it is not what a hit is either: the same twelve units are inside 大玉's own half and
        // outside 米弾's, so a slowing taken off by a distance is one taken off for one bullet and not the
        // other with nothing to tell them apart.
        game.image()
            .a_bullet_is_at(500, MIDDLE + A_STEP * 3.0, LOW, (0.0, 0.0));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            FOCUS,
            "the bot gave up the damage for a bullet that was not going anywhere:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And one that reaches the move being made takes it off, hit or not**: a dodge at half a step is half
        // a dodge, and by the frame a bullet is a hit the halving has already been paid for.
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - A_STEP * 6.0, (0.0, A_STEP));
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

/// **A bullet as wide as 妖々夢's widest has to be moved for before it is a step away.**
///
/// The frames of warning a move needs are `(the bullet's half + the player's half) / a step`: to be out from
/// under something [`WIDEST_BULLET`] across the player has to have travelled half of that plus its own half,
/// and a step is [`A_STEP`] — so a bullet first seen three frames out is one there is no getting away from,
/// whatever is pressed. It is not the bullet's own speed that sets that number, the frames being counted to the
/// moment it arrives and each bullet's own step being what the projection is made of; the speed only says how
/// far off it is when the frames run out.
///
/// So this puts one six frames from the player and asks for a move. Standing still is the answer that has to
/// stop being given.
#[test]
fn the_bot_moves_for_a_wide_bullet_before_it_is_three_frames_off() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-sees-a-wide-bullet-coming");
        game.image().the_player_is_at(MIDDLE, LOW);
        // A little to the side, so that there is a way out to be found and not only a way to be hit.
        game.image().a_bullet_of_size_is_at(
            0,
            MIDDLE + 10.0,
            LOW - 60.0,
            (0.0, 8.0),
            (WIDEST_BULLET, WIDEST_BULLET),
        );

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot held still under a bullet {WIDEST_BULLET} across and six frames off:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A move that clears by less than [`ROOM_TO_SPARE`] is not a move that clears.**
///
/// Which the launches say and the arithmetic does not: three hits in one stage read `0.6`, `1.1` and `0.6` of
/// room, so on each of them the model had the hazard, had its size, and put the player past it by less than a
/// unit — and the game hit anyway. Something in the projection is out by about a unit and no reading has said
/// what; until one does, room under a unit is not room. The game's own test helps it along, comparing edges
/// inclusively at 0x43e2e7 where this one is strict, so nought of room is already a hit there.
///
/// **And the choice needs it as much as the test does**: among moves that clear, the one taken is the nearest to
/// where the rules want to be, and nothing weighed the room — so a move shaving a bullet by half a unit beat one
/// with the whole field of it for being four units closer to an item. This is what stops that.
#[test]
fn a_move_that_only_just_clears_is_not_taken() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-room");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Something to want, across the field and level, so that straight across is the move to beat.
        game.image().an_item_is_at(700, POWER, 380.0, LOW);
        // And a bullet the straight move slips past by one unit: level with the path by its box on the y, and
        // crossed on the x somewhere along it.
        game.image()
            .a_bullet_is_at(0, MIDDLE + 20.0, LOW + A_STEP + 1.0, (0.0, 0.0));

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot shaved a bullet by a unit to get four units nearer an item:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The room a move has is measured between the boxes and not between the middles**, and so is which hazard
/// had the least of it.
///
/// A hit is two boxes overlapping, so what says how near a move came to being one is what was left on the
/// tighter of the two axes — and a distance between middles is not that: a 米弾 six units away has two units of
/// room and a 大玉 seventeen away has one, and the middles put them the other way round. Which is not only a
/// number in a log: `nearest was a bullet …` is what a hit is read against, and a line naming a bullet the
/// player was drifting away from is a line that sends whoever reads it to the wrong part of the model. One did:
/// a hit at `159.4,385.3` blamed a bullet at `152.9,387.6` **stepping downwards, below the player, while the
/// player pressed up**.
#[test]
fn the_room_a_move_has_is_measured_between_the_boxes() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-room-is-between-the-boxes");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Near by its middle and roomy by its box: no size of its own, so all that stands between them is the
        // player's own half. Four of room, which is over [`ROOM_TO_SPARE`] and so still a place to stand.
        game.image()
            .a_bullet_is_at(0, MIDDLE + A_STEP * 2.0, LOW, (0.0, 0.0));
        // And far by its middle and three of room by its box, which is less.
        let big = MIDDLE + A_STEP + WIDEST_BULLET / 2.0 + 3.0;
        game.image().a_bullet_of_size_is_at(
            1,
            big,
            LOW,
            (0.0, 0.0),
            (WIDEST_BULLET, WIDEST_BULLET),
        );

        game.frames(1);
        game.image().the_player_has_been_hit();
        game.frames(1);
        assert!(
            game.log().said(&format!(
                "with 3.0 of room — nearest was a bullet at \
                 {big:.1},{LOW:.1} stepping 0.0,0.0, half of it {:.1},{:.1}",
                WIDEST_BULLET / 2.0,
                WIDEST_BULLET / 2.0,
            )),
            "the hit named the bullet whose middle was nearest rather than the one whose box was:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **With nothing open, the move taken is the one whose trouble is furthest off** — and not the one that runs
/// into least of it.
///
/// Because the move is made again next frame. What a move has to buy is frames to decide in, so a move clear
/// for nine of them and hit on two after that is worth more than one hit once on the very next frame — and
/// counting the frames lost puts those the other way round. It is [`WIDEST_BULLET`] that makes this matter:
/// looking three frames out, "hit at all" and "hit next frame" are nearly the same thing, and looking far
/// enough ahead to leave a 大玉 they are not. A launch that looked twelve frames out and still ranked by the
/// count was hit nine times before 3面's boss, where the launch before it had reached 4面 without a scratch.
///
/// So this shuts all nine moves and leaves two of them apart only in when: left is hit on the second frame,
/// right on the tenth and eleventh.
#[test]
fn the_move_taken_is_the_one_whose_trouble_is_furthest_off() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-buys-frames");
        game.image().the_player_is_at(MIDDLE, LOW);
        // A box and a step far apart, so that one landing of the nine can be shut without shutting the ones
        // either side of it: at the ordinary step every landing is inside its neighbour's [`ROOM_TO_SPARE`].
        let (step, across) = (A_STEP * 4.0, A_STEP * 4.0 / 2.0_f32.sqrt());
        game.image().the_player_steps_by(step, across);
        game.image().the_players_box_is((1.0, 1.0));

        // Seven of the nine shut on the very first frame looked at, each by a bullet sitting where that move
        // lands after one frame of holding it.
        for (slot, (dx, dy)) in [
            (0.0, 0.0),
            (0.0, -step),
            (0.0, step),
            (-across, -across),
            (across, -across),
            (-across, across),
            (across, across),
        ]
        .into_iter()
        .enumerate()
        {
            game.image()
                .a_bullet_is_at(slot, MIDDLE + dx, LOW + dy, (0.0, 0.0));
        }
        // Left, shut on the second frame looked at and clear on every other.
        game.image()
            .a_bullet_is_at(7, MIDDLE - step * 2.0, LOW, (0.0, 0.0));
        // And right, shut on the tenth and the eleventh.
        game.image()
            .a_bullet_is_at(8, MIDDLE + step * 10.0, LOW, (0.0, 0.0));
        game.image()
            .a_bullet_is_at(9, MIDDLE + step * 11.0, LOW, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot took the move it ran into least of rather than the one that lasted:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy is moved for by where it is going and not only by where it stands.**
///
/// Nothing in an enemy's own update is a step a read could take — its script moves it — so the only way to
/// know is to remember where it was last frame, which is the way a laser's turn is known. A body taken for a
/// still thing is a body the model says there is room beside while it closes: a launch was hit beside 2面's
/// midboss with `4.7 of room`, which is a miss by less than the body moves in a frame.
#[test]
fn an_enemy_is_moved_for_by_where_it_is_going() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-sees-an-enemy-closing");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .an_enemy_of_size_is_at(0, MIDDLE, LOW - 60.0, (24.0, 24.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for a body that was standing still well clear of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // One frame on and eight units nearer, which is the whole of what says it is coming: from there it
        // reaches the player in six frames, and a body read as standing still never arrives at all.
        game.image()
            .an_enemy_of_size_is_at(0, MIDDLE, LOW - 52.0, (24.0, 24.0));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot held still under a body closing on it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A boss's slot is not cleared when that boss leaves**, so a slot holding a pointer is not a boss to stand
/// under — the enemy it leads to has to still be alive.
///
/// 0x4232a0 walks the eight slots and answers 1 at the first that is not null, and nothing else in the exe so
/// much as touches them. 妖々夢's 4面 is where that costs a run: three bosses share the stage, and a launch
/// spent 22505 of its 38820 frames pinned against the left wall with the x it wanted read off a slot whose
/// boss had gone.
#[test]
fn a_bosss_slot_left_behind_is_not_a_boss_to_stand_under() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-stale-boss");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().a_boss_slot_leads_to(0, 0);
        game.image().an_enemy_is_at(0, -50.0, 100.0);
        game.image().the_enemy_is_gone(0);
        game.image().a_boss_slot_leads_to(1, 1);
        game.image().an_enemy_is_at(1, 300.0, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot went for the boss that had left:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a boss outside the play field is not one either.** The x of something the player cannot get under is
/// an x it walks into a wall for, and a wall is where a dodge runs out of moves — so the middle is a better
/// place to be than under a boss that is not there to be stood under.
#[test]
fn a_boss_outside_the_play_field_is_not_stood_under() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-boss-off-the-field");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().a_boss_slot_leads_to(0, 0);
        game.image().an_enemy_is_at(0, FIELD.0 - 50.0, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot walked into the wall for a boss beyond it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **低速移動 is for the fight and not for the collecting**: the same key that puts the damage up halves the
/// step, and an item most of a field away is a second of the fight spent crossing it.
#[test]
fn an_item_takes_the_bot_out_of_low_speed() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-hurries-for-an-item");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(MIDDLE, 100.0);
        game.image().the_player_is_at(MIDDLE, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            FOCUS,
            "the bot fought a boss at full speed with nothing to collect:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().an_item_is_at(700, POWER, 380.0, 20.0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & FOCUS,
            0,
            "the bot went for an item at half speed:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
