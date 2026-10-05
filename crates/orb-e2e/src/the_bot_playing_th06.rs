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

use crate::fake::th06::{A_BOMB_IS_TAKEN_FOR, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Boss;

/// th06's own buttons, as `Th06` has them: the shot, the bomb, and the four of the d-pad.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const FOCUS: u16 = 0x004;
const PAD: u16 = UP | DOWN | LEFT | RIGHT;

/// The kinds of item this game drops, as the table at 0x4200f8 indexes them.
const POWER: i8 = 0;
const POINT: i8 = 1;
const BIG_POWER: i8 = 2;
const LIFE: i8 = 5;
/// And the power this game stops adding to, as `th06::POWER_FULL` has it.
const FULL_POWER: u16 = 128;

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

/// **The focus key is held under a boss where the move chosen has room to spare, and let go where it has only
/// the slack.**
///
/// **Which is what the halved step costs.** The focused pair is half the whole one in both of these games, so
/// a frame whose margin is a slack and no more is a frame that wants the ground the whole step covers — and
/// four hits of one th06c Extra launch were exactly that frame: each a move the model called clear with
/// `2.3`, `2.4`, `2.7` or `4.2` of room and the key held.
///
/// **The bullet here reaches none of the nine** — two steps off on each axis with no step of its own — so
/// every move stays clear, and what changes between the two halves of this is only how much room the move
/// chosen is left with: `4` against the `6` `steer::holds_the_focus` asks for.
#[test]
fn the_focus_key_is_held_only_where_the_move_chosen_has_room_to_spare() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-focuses-with-room-to-spare");
        game.image().boss(Some(Boss {
            life: 1000,
            attack_frames: 0,
        }));
        game.image().the_boss_is_at(MIDDLE, 96.0);
        game.image().the_player_is_at(MIDDLE, LOW);

        game.frames(3);
        assert_eq!(
            game.image().input_now() & FOCUS,
            FOCUS,
            "the bot did not hold the focus key with a boss up and every move wide open:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image()
            .a_bullet_is_at(0, MIDDLE + 2.0 * A_STEP, LOW - 2.0 * A_STEP, (0.0, 0.0));
        game.frames(3);
        assert_eq!(
            game.image().input_now() & FOCUS,
            0,
            "the bot held the focus key on a move left with only the slack:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where the whole step finds nothing open and the fine one does, the fine one is taken and the focus key
/// pressed.**
///
/// **Because a move is judged on a path and not a landing.** `LOOK_AHEAD` frames of the whole step travel
/// thirteen times it and of the focused one half that, so a wall the long path walks into is one the short
/// path stops short of — which is the frame this is about: the ring below is `100` units out, where
/// `steer::LOOK_AHEAD` whole steps reach `160` and as many focused ones `80`.
///
/// **And standing still has to be shut for the frame to be one at all**, which the bullet overhead does: a
/// frame with a way out that stays put is a frame the whole step already had.
///
/// **What the launches this is for look like**: three bombs of one th06c Extra attempt went on the fan its
/// boss lays — nine lanes `0.039` rad apart from one origin, `0.023` rad of lane and `4.3` units of gap
/// between two at the player's own radius — each with `the model was holding 0 within reach`, no bullets at
/// all and every one of the nine shut.
#[test]
fn the_fine_step_is_taken_where_the_whole_one_finds_nothing_open() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-takes-the-fine-step");
        // **A player small enough for the ring to be a ring and not a wall**, the slack a move is kept with
        // being two units on top of both halves.
        game.image().the_players_box_is((1.0, 1.0));
        game.image()
            .the_player_steps_by_focused(A_STEP / 2.0, A_STEP / (2.0 * 2.0_f32.sqrt()));
        // **In the middle of the field and not on the line the bot waits on**, which the ring's own size
        // decides: it has to surround the player, and a hazard outside the play field is one the model leaves
        // out — so a ring wider than the ground below the waiting line has its lower half dropped and the
        // downward moves come out open. `steer::LOOK_AHEAD` frames of the focused step cover `80` where that
        // line leaves `64` beneath it, and the middle leaves half the field either way.
        let deep = FIELD.1 + FIELD.3 / 2.0;
        game.image().the_player_is_at(MIDDLE, deep);
        // Eight of them at the eight ways out, and the ninth overhead so that standing still is shut too.
        let ring = 100.0_f32;
        for (slot, (dx, dy)) in [
            (-1.0_f32, -1.0_f32),
            (0.0, -1.0),
            (1.0, -1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, 1.0),
            (0.0, 1.0),
            (1.0, 1.0),
        ]
        .into_iter()
        .enumerate()
        {
            // **On the circle and not on the square its eight corners make**, which is what the whole step
            // has to be shut by: at the square's corners the four diagonals stand `141` out where the four
            // straights stand `100`, and the diagonal steps reach `113` — so the diagonals would come out
            // open by `28` and the frame would have a way out at the whole step after all.
            let out = ring / (dx * dx + dy * dy).sqrt();
            game.image()
                .a_bullet_is_at(slot, MIDDLE + dx * out, deep + dy * out, (0.0, 0.0));
        }
        // **And the ninth walked down by its own step every frame**, which is what makes it a bullet the
        // rules project rather than one a script is moving: `steer::A_STEP_OF_ITS_OWN` reads a bullet that
        // claims a step and has not moved as a bullet something else is carrying, and fears it over the
        // nothing it just moved — so a bullet placed once and left there leaves standing still open, and
        // the frame this is about never arrives.
        //
        // **And left far enough above that the fine step gets out from under it**, which the ring decides
        // the other end of: it falls onto the player's row eight frames out, by when a focused move is `16`
        // to the side and a whole one `32` — so standing still is shut by it either way, and what the fine
        // step is left with is the `20` between its own reach and the ring.
        //
        // **And the player put back in the middle of the ring every frame**, which is what makes the ring a
        // ring: the rules move it on the first of these frames, and one step off centre is a player the ring
        // no longer surrounds — standing still then reads `3.0` of room off the lane the bullet overhead has
        // left, and the frame is not the frame any more.
        for frames in 0..3 {
            game.image().the_player_is_at(MIDDLE, deep);
            game.image().a_bullet_is_at(
                8,
                MIDDLE,
                deep - (8.0 - frames as f32) * A_STEP,
                (0.0, A_STEP),
            );
            game.frames(1);
        }
        assert_eq!(
            game.image().input_now() & FOCUS,
            FOCUS,
            "the bot did not press the focus key on a frame the whole step found nothing open on:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A recipe that names the item to answer the retry question with gets to answer it.**
///
/// What a press reaches at that menu is the item the cursor is on, which is the chapter that was lost — so a
/// launch playing itself could take that one and nothing else, and the stage's own start was the whole run
/// played again to reach. See `retry_ui::RetryMenu::answered_outright`.
///
/// **The item is the stage's place in the list as the list stands**: a run at a stage's own first chapter has
/// nothing behind that chapter, so the item that lists what is behind it is left out and the stage is the
/// second.
///
/// **And the front end pressed through by the recipe's own first step**, which is what a launch playing to one
/// does: the bot answers every read of the game's own keyboard, so the presses the walk-in wants have to be
/// the bot's — a step that waits presses nothing, and the walk-in then waits for a decide nobody makes.
#[test]
fn a_recipe_of_th06_answers_the_retry_question_with_the_item_it_names() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-bot-answers-the-retry", the_run(), |config| {
            config.autoplay = Some(
                "- press: [decide]\n  frames: 2\n  until: { stage: 1, chapter: 1 }\n  \
                 give_up_after: 1800\n\
                 - start_over: true\n\
                 - answer: { question: retry, item: 1 }\n"
                    .to_owned(),
            );
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();
        game.frames_until("the question the start-over raises", 600, || {
            log.said("where from is the question")
        });
        // **The stage taken outright and not asked about**, which is what the run being at that stage's own
        // first chapter makes of it: the confirmation is for a choice that loses something, and a stage whose
        // start is the chapter being played loses nothing.
        game.frames_until("the stage taken", 600, || {
            log.said("retry: the stage again")
        });
        assert!(
            !log.said("retry: the chapter again"),
            "the retry was answered with the chapter the run was in, which is the item the recipe said it \
             did not want:\n  {}",
            log.lines().join("\n  "),
        );
    });
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
/// **What the rules make of a warning line is not read back here**, and the frames until one hits are why: they go
/// across as they are read — the frame of the warning it starts hitting on less the frames it has run — and a
/// beam is left whether it is firing or about to, which is `the_bot_playing_th07`'s subject and the same rules
/// over these numbers.
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

/// **A beam is left where it is being carried to and not where it is**, which is the reading nothing on a beam
/// carries: what says where one is going is the distance it covered last frame, and a beam taken as standing
/// still is a lane judged clear for every frame looked at while the boss walks it onto the player.
///
/// The player is put clear of the beam and off the x it wants to wait at, so that the move it makes with the
/// beam standing still is towards the beam and not away from it: the answer read back is which of those it did.
#[test]
fn the_bot_leaves_where_a_beam_of_th06_is_carried_to() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-leaves-a-carried-beam");
        let carried = A_STEP / 2.0;
        let clear_of_it = A_STEP * 4.5;
        // Straight down the whole field, a frame short of where it will be read from.
        game.image().a_laser_is_at(
            10,
            (MIDDLE - carried, 0.0),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            16.0,
        );
        game.image().the_player_is_at(MIDDLE + clear_of_it, LOW);
        game.frames(1);

        // And the frame the carry is read from: the beam a half step on, with the player put back where it was.
        game.image().the_laser_was_carried_to(10, (MIDDLE, 0.0));
        game.image().the_player_is_at(MIDDLE + clear_of_it, LOW);
        game.frames(1);
        // The x and not the whole word: a beam straight down the field is left by moving across it, and which
        // way the move goes up or down is what the place it wants to wait at decides.
        assert_eq!(
            game.image().input_now() & (LEFT | RIGHT),
            RIGHT,
            "the bot did not leave the lane a beam was being carried into:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the band a row of lanes is being laid across is left, and not only the lane nearest the player.** Two
/// lanes a frame apart in their own phases are one step of one sweep, and the step is how far apart they
/// stand: the lane after them is that far on again, and every across from here to where the sweep has reached
/// is a place to be out of.
///
/// The newer of the two is put clear of the player by more than a move is kept with, and the older behind it,
/// so the pair taken as two standing lanes leaves every one of the nine open and the sweep shuts all but the
/// three that go the other way — which is not where the rules want to be, the middle of the field being back
/// through the band. The answer read back is which of those the run was handed.
///
/// **Two lanes and not one**, which is what makes this the rule it is about rather than
/// [`the_bot_leaves_where_a_beam_of_th06_is_carried_to`]'s: a beam whose own slot says where it came from is
/// projected along that, and one lane by itself says nothing at all about where the next is laid.
#[test]
fn the_bot_leaves_the_band_a_row_of_lanes_of_th06_is_being_laid_across() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-leaves-a-laid-band");
        // **A step of the sweep smaller than a step of the player's**, so that going the way the sweep is
        // heading stays clear and the frame has an answer: `2.0` against the `4.0` of a whole step.
        let apart = A_STEP / 2.0;
        let across = 3.0;
        // Clear of the newer lane by more than the two halves and the slack — `14` against the `3` of the
        // lane, the `4` of the player and the `2` a move is kept with — and inside the band the sweep lays
        // over the frames looked at.
        let clear_of_it = 14.0;
        // Straight down the whole field, both of them: the older behind, the newer a step of the sweep on.
        for (slot, x) in [(10, MIDDLE), (11, MIDDLE + apart)] {
            game.image().a_laser_is_at(
                slot,
                (x, 0.0),
                std::f32::consts::FRAC_PI_2,
                (0.0, FIELD.3),
                across * 2.0,
            );
        }
        // And the older one a frame further into its phase, which is the whole of what says it was laid first.
        game.image().the_laser_was_carried_to(10, (MIDDLE, 0.0));
        game.image()
            .the_player_is_at(MIDDLE + apart + clear_of_it, LOW);

        game.frames(1);
        assert_eq!(
            game.image().input_now() & (LEFT | RIGHT),
            RIGHT,
            "the bot did not leave the band a row of lanes was being laid across:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And no power item at all where the shot is already full**, which is this game's own difference: th07
/// turns one into something else at full power and hands it over as that, where th06 leaves it a power item
/// worth a scrap of score — so a run that crossed the field for one crossed it for nothing.
///
/// Both kinds, the big one as much as the small: what either is worth is the power it adds.
#[test]
fn the_bot_goes_for_no_power_item_of_th06_at_full_power() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-leaves-power-at-full");
        // The two kinds of power item to one side and a point item to the other, so the answer is which way it
        // goes and nothing else.
        game.image().an_item_is_at(100, POWER, MIDDLE - 60.0, LOW);
        game.image()
            .an_item_is_at(200, BIG_POWER, MIDDLE - 40.0, LOW);
        game.image().an_item_is_at(300, POINT, MIDDLE + 60.0, LOW);
        game.image().set_power(FULL_POWER - 1);

        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            LEFT,
            "the bot passed over a power item with power left to gain:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().set_power(FULL_POWER);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            RIGHT,
            "the bot went for a power item with the shot already full:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And with only power items on the screen there is nothing to go for at all, so the rules stand where
        // they want to be rather than crossing to one.
        game.image().the_item_is_gone(300);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            0,
            "the bot crossed to a power item with nothing else on the screen:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A beam that is still growing is left before its end arrives**, which is the one thing a beam of this game
/// does with a frame: the far end reaches further every frame it fires and the near end follows.
///
/// **Which a run paid for.** Stage 2's boss fires icicles straight down at the player and lets them extend; a
/// launch stood five units under a far end, where every frame looked at read as clear, and died there nineteen
/// times over. So the growth goes across with the ends, and here it is read back: a beam that stops short and
/// stays there is nothing to move for, and the same beam growing is.
#[test]
fn the_bot_leaves_a_beam_of_th06_before_its_growing_end_arrives() {
    in_its_own_process(|| {
        let game = in_a_stage("th06-the-bot-leaves-a-growing-beam");
        // Straight down the middle from the top, ending a few units above where the player waits — which is
        // where the icicles that killed that run ended.
        game.image().a_laser_is_at(
            12,
            (MIDDLE, 0.0),
            std::f32::consts::FRAC_PI_2,
            (0.0, LOW - 8.0),
            32.0,
        );

        game.frames(1);
        assert_eq!(
            game.image().input_now() & PAD,
            0,
            "the bot moved for a beam that stops short of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The same beam reaching four further every frame, which is under the player inside the frames the
        // rules look at.
        game.image().the_laser_grows_by(12, 4.0);
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        assert_ne!(
            game.image().input_now() & PAD,
            0,
            "the bot stood under a beam that was growing onto it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the bomb on the frame the player is hit**, which is a deathbomb: `Player::Die` leaves the state at 2
/// for the frames a bomb is still taken in, and one pressed inside them takes the death back.
///
/// **The hit is a bullet's and not a state written by hand**, which is what makes the rest of this an
/// assertion about anything: the count moving is what orb reads a death off, so a state written straight into
/// the image would leave `runtime` with nothing to decide and the last assertion here true either way.
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

        // The hit itself, and the frame the game's own test takes it on.
        let lives = game.state().lives;
        game.puts_a_bullet_on_the_player();
        game.frames(1);
        assert_eq!(
            game.state().deaths,
            1,
            "the bullet did not reach the player at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And the frame after it, which is the word the rules answer a hit with**: they are asked for one
        // before the game's own update, so the bomb goes in inside the same frame it is pressed for.
        game.frames(1);
        assert_eq!(
            game.image().input_now() & BOMB,
            BOMB,
            "the bot did not bomb the frame the player was hit:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And the game takes it**, which is what the frames a bomb is taken in are for: the bomb goes in
        // while `respawnTimer` still stands, the state it leaves is the invulnerable one, and the run plays
        // on with the life it had and no offer to put a chapter back.
        assert_eq!(
            game.image().player_now(),
            orb_core::game::th06::image::Player::Invulnerable,
            "the bomb did not take the death back:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            game.state().lives,
            lives,
            "the death the bomb took back cost a life anyway:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 1);
        assert!(
            !game.log().said("died in chapter"),
            "a chapter was offered back for a death the bomb had taken:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
