//! **orb playing th06 itself**, which is the same rules over another game's arrays.
//!
//! What the rules *are* is `the_bot_playing_th07`'s subject and is not written twice: they are `orb-bot`'s and
//! no game's. What is here is the reading — th06's own field, its player, its bullets, its enemies, its items
//! and its lasers — and each of these says that one part of it arrived, by what the rules then do about it.
//!
//! **Which is the only way a reading can be checked at all.** An offset written down against a listing is a
//! claim about the exe; a run whose bot steps out from under a bullet placed in slot 300 is that claim carried
//! out. So every one of these puts something where the game would keep it and reads back the word the run was
//! handed.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;

/// th06's own buttons, as `Th06` has them: the shot, the bomb, and the four of the d-pad.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const PAD: u16 = UP | DOWN | LEFT | RIGHT;

/// The kinds of item this game drops, as the table at 0x4200f8 indexes them.
const POWER: i8 = 0;
const LIFE: i8 = 5;

/// A field for these to move in, and the places in it the rules are read against: the y they wait at, off the
/// bottom edge, and the middle of it.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
const LOW: f32 = 384.0;
const MIDDLE: f32 = 192.0;
/// What one frame of holding a direction moves the player by in these, straight — and a diagonal's own share
/// of it, a diagonal covering the corner of the square a straight move covers the side of.
const A_STEP: f32 = 4.0;

/// **A run of this game being played by the rules**, with the field and the player put where every one of
/// these reads them.
///
/// The whole walk in first, because the reading is gated on a stage running: `sight::in_a_stage` asks the
/// supervisor's own scene and the two menu flags, so a game that had not started a run answers the bot with
/// nothing at all.
fn in_a_stage(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        // A recipe of no steps, which is a run played from its first frame: what these are about is the
        // reading and not the screens on the way in.
        config.autoplay = Some("[]".to_owned());
        config.log_level = LogLevel::Verbose;
    });
    game.in_a_pointdevice_run();
    game.image()
        .the_play_field_is(FIELD.0, FIELD.1, FIELD.2, FIELD.3);
    game.image()
        .the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
    // And a box for it to be hit in, so that these read as "nearer than a step on both axes and it is a hit".
    game.image().the_players_box_is((A_STEP, A_STEP));
    game.image().the_player_is_at(MIDDLE, LOW);
    game
}

/// **The shot is held on every frame of a stage**, which says the frame reached the rules at all: off a stage
/// they press nothing, so a word with the shot in it is a stage the reading found.
#[test]
fn the_bot_holds_the_shot_down_in_a_stage_of_th06() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-shoots");

        game.frames(30);
        assert_eq!(
            game.image().input_now() & SHOT,
            SHOT,
            "the bot played a frame of a stage without the shot held:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The player is held low in the field the game says it may move in**, which is the field read off
/// `playerMovementAreaTopLeftPos` and its size rather than the one the bullets bounce in.
#[test]
fn the_bot_keeps_the_player_low_in_th06s_own_field() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-stays-low");
        game.image().the_player_is_at(MIDDLE, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            DOWN,
            "the bot left the player high up the field:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_player_is_at(MIDDLE, FIELD.3);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            UP,
            "the bot left the player against the bottom edge:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet coming down onto the player is stepped out from under**, which is the bullet array arriving: the
/// base, the stride, the state the walk skips a slot on, the position and the velocity a frame moves it by.
///
/// The slot is one well into the array, since what a walk answers is *which* slot a thing is in.
#[test]
fn the_bot_steps_out_from_under_a_bullet_of_th06() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-dodges-a-bullet");
        game.image()
            .a_bullet_is_at(300, MIDDLE, LOW - 24.0, (0.0, 8.0));

        game.frames(1);
        assert_ne!(
            game.image().input_now() & PAD,
            0,
            "the bot stood under a bullet on its way onto it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same slot emptied, which is the state the walk skips: nothing to dodge, so the rules stay
        // where they want to be — which is where the player is put back to first, the frame above having
        // moved it.
        game.image().the_bullet_is_gone(300);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            0,
            "the bot dodged a slot with no bullet in it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And a bullet the player has already been hit by is dodged all the same**, which is the rules'
        // doing and not the reading's: what the game says of a state goes across, and th07 paid for leaving
        // a state out — see `the_bot_playing_th07`.
        game.image()
            .a_bullet_is_at(300, MIDDLE, LOW - 24.0, (0.0, 8.0));
        game.image().the_bullet_is_spent(300);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_ne!(
            game.image().input_now() & PAD,
            0,
            "the bot stood under a bullet the game had spent:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy's body is dodged as much as a bullet**, its body being as solid as one in this game — and one
/// with nothing solid on it is not, the game's own collision never being run for it.
#[test]
fn the_bot_dodges_a_body_of_th06_and_not_one_with_nothing_solid_on_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-dodges-a-body");
        // Standing on the player, wide enough that its own box reaches: an enemy is dodged where it stands,
        // there being no step on one to project.
        game.image()
            .an_enemy_of_size_is_at(200, MIDDLE, LOW, (48.0, 48.0));

        game.frames(1);
        assert_ne!(
            game.image().input_now() & PAD,
            0,
            "the bot stood inside an enemy's own body:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_enemy_has_no_body(200);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            0,
            "the bot dodged a body the game would not have tested it against:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The items are gone for, and the one worth most first**: the array, the stride and the kind, which is what
/// says a 1up is a 1up and not a power item.
///
/// Both level with the player and either side of it, so the answer is left or right and nothing else.
#[test]
fn the_bot_goes_for_th06s_item_worth_most() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-collects");
        game.image().an_item_is_at(100, POWER, MIDDLE - 60.0, LOW);
        game.image().an_item_is_at(400, LIFE, MIDDLE + 60.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            RIGHT,
            "the bot went for the power item over the 1up:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The 1up collected out of its slot, which is the alive byte cleared: what is left is the power item.
        game.image().the_item_is_gone(400);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            LEFT,
            "the bot did not go for what was left after the 1up:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A laser that is firing is left**, which is the laser array arriving: its own slots, the word that says a
/// slot holds one, and the geometry the player's test is given.
///
/// **What the rules make of a warning line is not read back here**, and the frames until one fires are why: they go
/// across as they are read — the phase's own length less the frames it has run — and a beam is left whether it
/// is firing or about to, which is `the_bot_playing_th07`'s subject and the same rules over these numbers.
#[test]
fn the_bot_leaves_a_laser_of_th06_that_is_firing() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-leaves-a-laser");
        // Straight down the field through where the player stands, wide enough to be standing in.
        game.image().a_laser_is_at(
            10,
            (MIDDLE, 0.0),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            32.0,
        );

        game.frames(1);
        assert_ne!(
            game.image().input_now() & PAD,
            0,
            "the bot stood in a beam that was firing:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the slot freed, which is the word the manager's own walk skips a slot on: nothing left to be out
        // of the way of, so the rules stay where they want to be.
        game.image().the_laser_is_gone(10);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            0,
            "the bot left a beam its own manager had freed the slot of:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the bomb on the frame the player is hit**, which is a deathbomb: `Player::Kill` leaves the state at 2
/// and a bomb pressed while it stands takes the death back.
#[test]
fn the_bot_bombs_the_frame_th06s_player_is_hit() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-death-bombs");

        game.frames(4);
        assert_eq!(
            game.image().input_now() & BOMB,
            0,
            "the bot bombed a frame nothing had hit the player on:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And one frame, which is what this state lasts here**: the game's own update puts it back to normal
        // on the very frame it is read on, so the word that carries the bomb is this frame's and no later
        // one's.
        game.image()
            .player(orb_core::game::th06::image::Player::Dying);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & BOMB,
            BOMB,
            "the bot did not bomb the frame the player was hit:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
