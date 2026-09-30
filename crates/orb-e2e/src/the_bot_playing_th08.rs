//! **The rules playing a laid-out th08, over the two readings of a bullet this game's field alone does not
//! carry: the frame it is born on, and a step that is changing under it.**
//!
//! th06's and th07's own files hold the field arriving — the array, the stride, the slot skipped — and th08's
//! walk hands the same things over the same way. What is this game's own is what the step at `BULLET_STEP`
//! is *not*: a bullet read in kinds 2 to 4 is on its first frame and moves the step and a share of it again
//! on the next — a launch lost one fight fifty times over at one hit, each dump naming three fresh bullets
//! that had moved `2.99` saying `2.00` — and a bullet under the behaviour at `BULLET_BEHAVIOURS` has
//! something added to its step every frame before the step is added to its position, so the step read after
//! a frame is one addition short of the one about to be taken.

use crate::fake::th08::{BOMBS, FIRST_STAGE, Fake, LIVES, THE_FIGHTS_TRACK};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// th08's own four directions, as `Th08::button_bit` has them.
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const PAD: u16 = UP | DOWN | LEFT | RIGHT;
/// And its shot and its focus key.
const SHOT: u16 = 0x001;
const FOCUS: u16 = 0x004;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The place in the field the player is put, which is where the rules wait: the middle, at the low line —
/// **the field being the four the player is held inside**, 8 and 16 in from the area's corner and 368 by
/// 416, so the middle is 192 and the low line, sixty-four above that rectangle's floor of 432, is 368.
const MIDDLE: f32 = 192.0;
const LOW: f32 = 368.0;

/// Slots well into the array, since what a walk answers is *which* slot a thing is in — and one per bullet,
/// because the rules remember a slot from the frame before and read a bullet that stood still against the
/// step it claims as one a script is moving: feared where it is rather than projected, which is not the
/// reading under test.
const SLOTS: [usize; 3] = [300, 301, 302];

/// **A fresh bullet's step**: read in kind 2 it moves the step and half of it again on its next frame, so
/// at `3.2` a frame from ninety-six above it is twenty frames off — where the half the kind's branch adds
/// on its own, which is what was handed over before, put it sixty frames off and past what the rules look
/// at.
const FRESH_SLOT: usize = 303;

/// The class of item an erased bullet leaves, which `sight::worth` ranks as worth nothing — and which on this
/// game is its **time orb**, the count a stage's Last Spell is due on.
const A_SCRAP: u8 = 7;

/// **How far beside the player a bullet is put where what a case is about is the field not being clear**:
/// half of `steer::CLEAR_ENOUGH_TO_SWEEP`, so the field reads as one to stay on, and far enough that the
/// frame this is read on is not one the rules dodge.
///
/// **On the side the case's own item is not**, so that the way to the item is not the way to the bullet: put
/// to the right of the player, with the item a step to the right, the rules would not step towards it and the
/// case that holds a near item taken would be a case about the bullet.
const A_BULLET_BESIDE_THE_PLAYER: f32 = 48.0;
const A_FRESH_STEP: f32 = 3.2;
const A_FRESH_KIND: u16 = 2;
/// And the kind a bullet spends its life in, which is the one whose box test takes a life.
const A_HITTING_KIND: u16 = 1;

/// Half the narrowest of the three beam widths, which is how far across one a run has to get to be clear
/// of it — `Th08::BEAM_BEHAVIOURS`, whose first entry is the one these tests fire.
const A_BEAMS_HALF: f32 = 160.0 / 2.0;

/// **How far off the middle of a lane the player is put where a test is about which of two it leaves**,
/// which has to be some of the way across and not all: a run at the middle of both has the same walk out of
/// either, and the two sides of one lane are the same distance only from its middle.
const OFF_A_BEAMS_MIDDLE: f32 = 40.0;

/// **The slot the cells of a grid are born and killed in**, and the two places one of them is put: a step of
/// `12.0,20.8` apart, which is `24 * (cos 60°, sin 60°)` — one step of the lattice of `蘇活「生命遊戯」`, read
/// off the halves five hits of one Normal run named. Up at the top of the field, where a cell's own `8.0`
/// reaches nothing the player can be.
///
/// The first slot, since `Image::enemies_alive` is what makes one live and it fills the slots from there.
const A_CELLS_SLOT: usize = 0;
const A_GRIDS_CELLS: [(f32, f32); 2] = [(96.0, 96.0), (108.0, 116.8)];

/// **How a body of that stream goes**: down the diagonal at `9.9` a frame, which is `7.0,7.0` on each axis —
/// the `8.0,9.9` the run's own hits measured — with the speed kicked by `4.0` every other frame.
///
/// **The kick and not a turn, because a turn changes the two axes by turns.** A body turning at `w` radians a
/// frame has its motion changing by `speed * w * sin` on one axis and `* cos` on the other, so wherever one is
/// large the other is small: at a rate that clears half a unit — `A_STEP_OF_ITS_OWN`, which is what says a
/// body's motion is changing at all — the axis whose own motion is nearly nothing is the axis the fear does not
/// cover, and a box covering one axis of the field holds nothing. A kick along the diagonal changes both at
/// once, which is what an enemy whose script sets it a new speed does.
const A_BODYS_SPEED: f32 = 9.9;
const A_BODYS_KICK: f32 = 4.0;
const A_BODYS_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
/// Where each is born, which is the field's own top-left corner and not the middle of anything.
const A_BODYS_START: (f32, f32) = (16.0, 40.0);
/// **The field as the rules are handed it**, which is what says a body has crossed it — 368 by 416 from the
/// area's own corner, see `sight::field`.
const FIELD: (f32, f32) = (368.0, 416.0);
/// **Six slots, eight frames apart, and as many frames as the grid above is given.** A crossing takes some
/// forty-five frames, so six of them at that spacing leave no frame with the field empty: what the record of
/// a place lost is written after is a *stretch* of frames with nothing open, a second of them, and two bodies
/// with a gap between their passes let the count start again every sixth frame.
const A_STREAMS_SLOTS: usize = 6;
const A_STREAMS_FRAMES_APART: usize = 8;
const A_STREAMS_FRAMES: usize = 300;

/// **How many frames of that grid are played**, which is longer than the stretch the rules write a place lost
/// after *and* longer than the four seconds they hold one record apart from the next: a second of frames with
/// nothing open is what the record is written for, and the first of them cannot be written before frame 240.
/// The first frame of a slot has nothing to difference against either, so the fear this is about cannot
/// appear before the second.
const A_GRIDS_FRAMES: usize = 300;

/// **How far above the player the bullet starts, and the two steps it is read with.** At two a frame the
/// bullet is forty-eight frames off, past the thirty-nine the rules look ahead; read by the step it is
/// about to take, eight, it is a dozen frames off, which is a bullet to step out from under now.
const ABOVE: f32 = 96.0;
const A_SLOW_STEP: f32 = 2.0;
const GROWING_BY: f32 = 6.0;
const FOR_FRAMES: i32 = 60;

/// A run of this game being played by the rules, placed rather than walked to: what these are about is the
/// reading of one bullet and not the screens on the way in.
fn in_a_stage(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            // A recipe of no steps, which is a run played from its first frame.
            config.autoplay = Some("[]".to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.is_in_a_run();
    game.image().the_player_is_at((MIDDLE, LOW));
    game
}

/// **A bullet whose step is growing is stepped out from under as fast as it is about to move**, and one whose
/// growing has run its frames is read by the step it has.
#[test]
fn the_bot_reads_a_bullet_of_th08_by_the_step_it_is_about_to_take() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-reads-a-growing-step");

        // Slow and far: nothing to dodge, so the rules stay where they want to be.
        game.image().a_bullet_is_at(
            SLOTS[0],
            (MIDDLE, LOW - ABOVE),
            (0.0, A_SLOW_STEP),
            (4.0, 4.0),
        );
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot dodged a slow bullet forty-eight frames off:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The same bullet with its step growing by six a frame for the next sixty: what the next frame adds
        // is eight, a dozen frames to the player, so the rules read it arriving and move.
        game.image().the_bullet_is_gone(SLOTS[0]);
        game.image().a_bullet_is_at(
            SLOTS[1],
            (MIDDLE, LOW - ABOVE),
            (0.0, A_SLOW_STEP),
            (4.0, 4.0),
        );
        game.image()
            .a_bullet_accelerates_by(SLOTS[1], (0.0, GROWING_BY), 0, FOR_FRAMES);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot stood under a bullet whose step was growing onto it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the growing over — as many frames run as it was given — which is the step as it stands and the
        // bullet back to being forty-eight frames off. **Frames of nothing first**, with the player put back
        // where the dodge took them from, so that nothing of the dodge just chosen is being spent when the next
        // choice is judged.
        game.image().the_bullet_is_gone(SLOTS[1]);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(2);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        game.image().a_bullet_is_at(
            SLOTS[2],
            (MIDDLE, LOW - ABOVE),
            (0.0, A_SLOW_STEP),
            (4.0, 4.0),
        );
        game.image()
            .a_bullet_accelerates_by(SLOTS[2], (0.0, GROWING_BY), FOR_FRAMES, FOR_FRAMES);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot dodged a bullet whose growing had run out:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet coming out is nothing to move for, standing on the run or not**, and the same bullet in the
/// kind it spends its life in is moved for: only that kind's box test takes a life, and one erased as it
/// comes out is harmless for the rest of its life — see `Th08::BULLET_HITS`.
///
/// **Which is what makes stage 5's second card playable at all.** Read as solid, a mesh of 747 of them
/// coming out left the rules `nowhere open to it` in every box, and the run backed into the bottom-left
/// corner and died there; a person plays that card standing in the mesh.
#[test]
fn the_bot_of_th08_stands_where_a_bullet_is_coming_out() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-stands-in-a-mesh-coming-out");
        // Right on top of the player, which is where a bullet coming out is harmless.
        game.image()
            .a_bullet_is_at(FRESH_SLOT, (MIDDLE, LOW), (0.0, A_FRESH_STEP), (4.0, 4.0));
        game.image().the_bullet_is_of_kind(FRESH_SLOT, A_FRESH_KIND);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot moved for a bullet coming out, which takes no life:\n  {}",
            game.log().lines().join("\n  "),
        );
        // The same bullet in the kind whose test takes a life, coming down on the run from far enough
        // off to be a move and not a hit: gone from.
        game.image().a_bullet_is_at(
            FRESH_SLOT,
            (MIDDLE, LOW - ABOVE),
            (0.0, A_FRESH_STEP),
            (4.0, 4.0),
        );
        game.image()
            .the_bullet_is_of_kind(FRESH_SLOT, A_HITTING_KIND);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot stood under a bullet in the kind that kills:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet the script has hidden is moved for all the same**, although the game makes neither of the
/// player's tests of it while it is hidden: it is shown again where it has got to and tested from that frame,
/// so a run that walked into it is standing in it then — see `Th08::BULLET_UNTESTED`.
#[test]
fn the_bot_of_th08_keeps_off_a_bullet_the_game_has_hidden() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-keeps-off-a-hidden-bullet");
        // Coming down on the run in the kind whose test takes a life, and hidden.
        game.image().a_bullet_is_at(
            SLOTS[0],
            (MIDDLE, LOW - ABOVE),
            (0.0, A_FRESH_STEP),
            (4.0, 4.0),
        );
        game.image().the_bullet_is_hidden(SLOTS[0]);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot stood under a bullet the game had hidden, which is shown again where it gets to:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An item below the low line is waited for at the low line and not followed down**: it is a frame or two
/// from leaving the field, and the bottom edge is where a run has half its moves gone. An item beside the
/// line is gone for, which is the same rule's other half.
#[test]
fn the_bot_does_not_follow_an_item_of_th08_below_the_low_line() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-waits-at-the-low-line");
        // Below the line and to one side, so that going for it would press down as well as across.
        game.image().an_item_is_at((MIDDLE + 40.0, LOW + 56.0));
        game.frames(1);
        let pressed = game.image().the_word_the_keys_read() & PAD;
        assert_eq!(
            pressed & DOWN,
            0,
            "the bot followed an item down below the low line:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            pressed & RIGHT,
            0,
            "the bot did not go across for the item at all:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A body in a slot another enemy had last frame is feared over nothing**, which is what the count each
/// enemy carries at `Th08::ENEMY_FRAMES` is read for — see `Hazard::frames`. A body's own motion is the
/// difference between two frames, there being no step on one to read; across two enemies that difference is
/// the distance between them, and what the rules do with a body whose motion is changing is fear its box over
/// everywhere it could reach in the frames looked at.
///
/// **Which is what had stage 6's `蘇活「生命遊戯」` unplayable.** That card stands a grid of cells up and kills
/// and bears them every few frames, so its slots hold a different cell continually: five hits of one Normal
/// run named `an enemy ... half of it 2408.0,4164.9` on a field 384 by 448, with every one of the nine moves
/// `hit first on 0` and `-2367.7 of room`, and the run stood still and died 87 times in the chapter. Those
/// halves are the read `8.0` grown by one step of the grid's own lattice for forty frames.
#[test]
fn the_bot_of_th08_fears_no_body_in_a_slot_another_enemy_had() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-a-slot-given-to-another-enemy");
        // **Past the frame the laid-out game puts its own title up on**, which is what a test that plays
        // more than a second of a stage needs: that screen replaces the run this was placed in, and the
        // rules stop being asked for a word the moment it does.
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().enemies_alive(1);
        // One cell of that grid in the slot at a time, each new and one lattice step from the last, up at the
        // top of the field where its own `8.0` reaches nothing: feared over the distance between two of them
        // its box is 488 by 840, which is the field twice over, so every one of the nine moves is inside it.
        for at in A_GRIDS_CELLS.iter().cycle().take(A_GRIDS_FRAMES) {
            game.image().an_enemy_is_new_in_its_slot(A_CELLS_SLOT, *at);
            game.image().the_player_is_at((MIDDLE, LOW));
            game.frames(1);
        }
        // **And the stretch is what says so and not the press**: a run with nothing open to it stands where
        // it wants to be, which is what a run with nothing to dodge does too — so the two are told apart by
        // the record the rules keep of a place lost, written once a stretch of them has run.
        assert!(
            !game.log().said("nothing open"),
            "the bot was shut in by a grid of cells 272 above it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a bullet in a slot another one had is differenced against nothing either**, which is the same
/// reading for the other array — see `Th08::BULLET_FRAMES`.
///
/// A bullet does carry a step, and what the rules do with one that moved otherwise than its step says is fear
/// the box it swept: across two bullets of one slot that disagreement is the distance between them.
/// `狂符「幻視調律」` lays a lattice of bullets over the field and cancels it again and again, so its slots
/// hold a different bullet every few frames — and a hit inside that card read `a bullet at 383.4,296.9
/// stepping 0.0,0.0, half of it 2.0,2.0, feared out to 385.4,2.0`: a bullet claiming no step at all, feared
/// across the whole width of a field 384 wide, laid over the row the run was standing in.
#[test]
fn the_bot_of_th08_fears_no_bullet_in_a_slot_another_one_had() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-a-bullet-slot-given-away");
        game.its_title_menu_is_up();
        game.is_in_a_run();
        for at in A_GRIDS_CELLS.iter().cycle().take(A_GRIDS_FRAMES) {
            game.image()
                .a_bullet_is_new_in_its_slot(SLOTS[0], *at, (0.0, 0.0), (2.0, 2.0));
            game.image().the_player_is_at((MIDDLE, LOW));
            game.frames(1);
        }
        assert!(
            !game.log().said("nothing open"),
            "the bot was shut in by one lattice bullet 272 above it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A body going somewhere is feared over the change in its motion and not over the motion**, which is what
/// keeps a moving enemy from shutting all nine moves at once.
///
/// A body carries no step — nothing on one is a rate, its script being what moves it — so where it will be is
/// the difference between the last two frames, and what is not known about that is how much that difference
/// itself changed. Spread by the *motion* for each of the forty frames looked at, an enemy crossing the field
/// at `8.0,9.9` a frame reached the rules as a box `320.2,393.0` across, which holds every one of the nine
/// landings and holds them on the same frame: **eight attempts at the last stage's `秘術「天文密葬法」` died at
/// `8.0,362.3`**, the left wall, each pressing down into it with `nowhere open to it` and the dump naming an
/// enemy whose own half was `8.0,8.0`.
#[test]
fn the_bot_of_th08_fears_a_moving_body_over_the_change_in_its_motion() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-a-body-crossing-the-field");
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().enemies_alive(A_STREAMS_SLOTS);
        // **A stream of them crossing the field, the way that card stands its forty-nine up**: each is born
        // at the top-left, goes at [`A_BODYS_SPEED`] with its direction turning [`A_BODYS_TURN`] a frame, and
        // is born again in its own slot once it is off the field — which is what the game does with the slot,
        // and what `an_enemy_is_new_in_its_slot` says. Two of them, half a crossing apart, so that one is
        // always somewhere in the middle of the field with both halves of its motion large.
        let mut going: Vec<((f32, f32), usize)> = (0..A_STREAMS_SLOTS)
            .map(|slot| (A_BODYS_START, slot * A_STREAMS_FRAMES_APART))
            .collect();
        // One step of one of them: the diagonal at the frame's own speed, and back to the top-left as a new
        // enemy in the slot once it is off the field.
        let step = |place: &mut (f32, f32), lived: &mut usize| -> bool {
            let speed = match lived.is_multiple_of(2) {
                true => A_BODYS_SPEED,
                false => A_BODYS_SPEED + A_BODYS_KICK,
            };
            *lived += 1;
            *place = (
                place.0 + speed * A_BODYS_ANGLE.cos(),
                place.1 + speed * A_BODYS_ANGLE.sin(),
            );
            match (0.0..FIELD.0).contains(&place.0) && (0.0..FIELD.1).contains(&place.1) {
                true => true,
                false => {
                    *place = A_BODYS_START;
                    *lived = 0;
                    false
                }
            }
        };
        // The one that starts half a crossing in, walked there before the run is asked for a word.
        for (slot, (place, lived)) in going.iter_mut().enumerate() {
            for _ in 0..*lived {
                step(place, &mut 0);
            }
            *lived = 0;
            game.image().an_enemy_is_new_in_its_slot(slot, *place);
        }
        for _ in 0..A_STREAMS_FRAMES {
            for (slot, (place, lived)) in going.iter_mut().enumerate() {
                match step(place, lived) {
                    true => game.image().an_enemy_is_at(slot, *place),
                    false => game.image().an_enemy_is_new_in_its_slot(slot, *place),
                }
            }
            game.image().the_player_is_at((MIDDLE, LOW));
            game.frames(1);
        }
        assert!(
            !game.log().said("nothing open"),
            "the bot was shut in by a stream of bodies crossing the field:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A beam an enemy's own script is firing is stepped out of, body or no body.** Three behaviours an
/// enemy's context can hold test a box 590 long along the enemy's angle every frame, the widest of them
/// 288 across — the third way this game kills, beside a bullet's box and an enemy's body, and the one a
/// run at stage 4's boss died to twenty-eight times with `236.7 of room`: nothing in the bullets, nothing
/// in the lasers, and a bodiless enemy 27 units off. Here the enemy sits in the player's own column and
/// the beam points straight down, so the way out of a box 590 long is sideways; an enemy with no beam
/// aimed at all is nothing to move for.
#[test]
fn the_bot_steps_out_of_a_beam_an_enemy_of_th08_fires_from_its_script() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-steps-out-of-a-scripted-beam");
        // One enemy, alive with no body, over the player: nothing to move for while its script fires
        // nothing.
        game.image().enemies_alive(1);
        game.image().an_enemy_has_no_body(0);
        game.image().an_enemy_is_at(0, (MIDDLE, 100.0));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the bot moved for a bodiless enemy firing nothing:\n  {}",
            game.log().lines().join("\n  "),
        );
        // The same enemy's script running the narrowest of the three beams, pointing down and sitting at
        // the middle of it: the box reaches 295 either way along the column, so the player inside it has
        // nowhere below to go and moves aside.
        game.image()
            .an_enemy_fires_a_beam(0, (MIDDLE, LOW), std::f32::consts::FRAC_PI_2, 0);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        let pressed = game.image().the_word_the_keys_read() & PAD;
        assert_ne!(
            pressed & (LEFT | RIGHT),
            0,
            "the bot stood inside a beam an enemy's script was firing, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The field is traced on every frame of the card a launch named, and on no frame of another**: every
/// bullet with where it is, its size, its kind and whether its script hides it, and how far into the card the
/// boss's own clock is — which is what a card is measured by before a way of dodging it is written.
#[test]
fn the_field_of_th08_is_traced_through_the_card_a_launch_named() {
    in_its_own_process(|| {
        const NAMED: u32 = 105;
        const ANOTHER: u32 = 101;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-field-traced-through-a-card",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
                config.trace_the_field_in = Some(NAMED);
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image()
            .a_bullet_is_at(SLOTS[0], (MIDDLE - 64.0, 200.0), (0.0, 0.0), (8.0, 8.0));
        game.image()
            .a_bullet_is_at(SLOTS[1], (MIDDLE + 64.0, 150.0), (0.0, 0.0), (8.0, 8.0));
        game.image().the_bullet_is_of_kind(SLOTS[1], 2);

        game.image().a_card_is_up(ANOTHER as usize);
        game.frames(3);
        game.image().a_card_is_up(NAMED as usize);
        game.image().the_boss_has_attacked_for(0, 40);
        game.frames(3);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");

        let traced: Vec<_> = records
            .iter()
            .filter_map(|record| record.the_field.clone())
            .collect();
        assert!(
            !traced.is_empty(),
            "no record of the named card holds its field"
        );
        assert!(
            traced.iter().all(|field| field.card == NAMED),
            "a frame of another card had its field traced: {:?}",
            traced.iter().map(|field| field.card).collect::<Vec<_>>(),
        );
        let field = &traced[0];
        assert_eq!(
            field.at,
            (MIDDLE, LOW),
            "the field was traced with the player somewhere else"
        );
        assert!(
            field.card_frames >= 40,
            "the card's own clock was not traced: {}",
            field.card_frames
        );
        let kinds: Vec<(u32, u16)> = field
            .bullets
            .iter()
            .map(|bullet| (bullet.slot, bullet.kind))
            .collect();
        assert!(
            kinds.contains(&(SLOTS[1] as u32, 2)),
            "a bullet coming out was not traced in its own kind: {kinds:?}",
        );
        assert!(
            field.bullets.iter().any(|bullet| bullet.slot == SLOTS[0] as u32
                && bullet.at == (MIDDLE - 64.0, 200.0)),
            "a bullet was not traced where it was: {:?}",
            field.bullets,
        );
    });
}

/// **A hit is traced with what stood over it**, enemies and bullets in every kind, with the words the game's
/// own tests are gated on: a hit can come with nothing in the model, and what was there is then the whole
/// question — one a check answers off the trace's own fields.
#[test]
fn a_hit_of_th08_is_traced_with_what_stood_over_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-hit-traced-with-what-stood-over-it",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        game.is_in_a_run();
        game.image().enemies_alive(1);
        game.image().an_enemy_has_no_body(0);
        game.image().an_enemy_is_at(0, (MIDDLE, LOW));
        game.image()
            .a_bullet_is_at(SLOTS[0], (MIDDLE + 4.0, LOW), (0.0, 0.0), (4.0, 4.0));
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().the_player_has_been_hit();
        game.frames(2);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let over = records
            .iter()
            .find_map(|record| record.over_a_hit.clone())
            .expect("no record of the trace holds what stood over the hit");
        assert_eq!(over.at, (MIDDLE, LOW), "the hit was traced somewhere else");
        assert!(
            over.enemies.iter().any(|enemy| enemy.slot == 0
                && enemy.at == (MIDDLE, LOW)
                && enemy.flags & 0x4 == 0),
            "the bodiless enemy over the hit was not traced: {over:?}",
        );
        assert!(
            over.bullets
                .iter()
                .any(|bullet| bullet.slot == SLOTS[0] as u32 && bullet.at == (MIDDLE + 4.0, LOW)),
            "the bullet beside the hit was not traced: {over:?}",
        );
    });
}

/// **A familiar is flown through by the two runs whose player the game never tests against one**, and kept
/// clear of by every other: the body test at 0x42c290 jumps past its deadly box for a familiar where the run's
/// index is 0 or 4, focused or not. Here a solid familiar sits on the player, which the first team stays under
/// and the second steps away from.
#[test]
fn a_familiar_of_th08_is_nothing_to_the_runs_it_never_hits() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-a-familiar-passed-through");
        game.image().enemies_alive(1);
        game.image().an_enemy_is_a_familiar(0);
        game.image().an_enemy_is_at(0, (MIDDLE, LOW - 8.0));
        game.image().the_character_playing_is(0);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the first team moved for a familiar the game never tests it against:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.image().the_character_playing_is(1);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & PAD,
            0,
            "the second team stood under a familiar that takes its life:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a beam fired from an anchor that is not the one a warning is read off**, which is a beam all the
/// same: the behaviour lays its box from `position - anchor` along the enemy's angle and tests it with
/// nothing of the anchor asked (0x424730). Here the anchor is nothing, so the box starts at the enemy
/// itself, over the player's own column, and points straight down through the player.
#[test]
fn the_bot_steps_out_of_a_beam_of_th08_fired_from_an_anchor_of_its_own() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-steps-out-of-a-beam-anchored-its-own-way");
        game.image().enemies_alive(1);
        game.image().an_enemy_has_no_body(0);
        game.image().an_enemy_fires_a_beam_anchored_at(
            0,
            (MIDDLE, 100.0),
            std::f32::consts::FRAC_PI_2,
            (0.0, 0.0),
            0,
        );
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        let pressed = game.image().the_word_the_keys_read() & PAD;
        assert_ne!(
            pressed & (LEFT | RIGHT),
            0,
            "the bot stood inside a beam fired from an anchor of its own, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A run already inside a beam walks out of it**, which the nine moves cannot decide: a beam 160 across
/// is twenty frames of a whole step from its middle to its edge, so every move is hit over the frames they
/// are looked over and the judgement has nothing to choose between them. A launch died of exactly that,
/// `nowhere open to it` with the dump's own `the nearest beam slot 2, -4.2 outside its box`.
#[test]
fn the_bot_of_th08_walks_out_of_a_beam_it_is_standing_in() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-walks-out-of-a-beam");
        game.image().enemies_alive(1);
        game.image().an_enemy_has_no_body(0);
        // A beam down the player's own column, firing: the way out is across it and takes many frames.
        game.image()
            .an_enemy_fires_a_beam(0, (MIDDLE, LOW), std::f32::consts::FRAC_PI_2, 0);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames_until("the run clear of the beam", 240, || {
            (game.image().where_the_player_is().0 - MIDDLE).abs() > A_BEAMS_HALF
        });
    });
}

/// **And the lane it leaves is the one that is firing, not the deeper one only aimed.**
///
/// **Because this game arms its lanes long before it fires them, four at a time.** Stage 4's fight stands
/// four bodiless enemies up with an anchor apiece and fires them in turn, so a run told every armed lane is
/// a lane already testing it has four of them across the bottom of the field and nothing open anywhere: the
/// rules died 95 times at `91.4,432.0`, every retry of the chapter on the same frame, with all four of them
/// `half of it 80.0,80.0` in the dump and one of them 46 units inside. So the behaviour field is what says a
/// lane is live, the anchor says only where one will be, and a run inside both walks out of the live one
/// however deep it stands in the other.
#[test]
fn the_bot_of_th08_leaves_the_beam_that_is_firing_before_one_only_aimed() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-leaves-the-firing-beam");
        game.image().enemies_alive(2);
        game.image().an_enemy_has_no_body(0);
        game.image().an_enemy_has_no_body(1);
        // A column firing, which the player stands 40 inside, and a lane aimed along the low line, which it
        // stands at the very middle of: the aimed one is the deeper of the two by half its width.
        game.image()
            .an_enemy_fires_a_beam(0, (MIDDLE, LOW), std::f32::consts::FRAC_PI_2, 0);
        game.image().an_enemy_aims_a_beam(1, (MIDDLE, LOW), 0.0);
        game.image()
            .the_player_is_at((MIDDLE + OFF_A_BEAMS_MIDDLE, LOW));
        game.frames_until("the run clear of the column that is firing", 240, || {
            (game.image().where_the_player_is().0 - MIDDLE).abs() > A_BEAMS_HALF
        });
    });
}

/// **A beam aimed and not yet firing is stepped out of too**, which is the whole of the warning this
/// game's beam gives: the behaviour its context runs turns on the frame the box first tests the player,
/// so a run inside it then is a run already hit. What says one is coming is the anchor — the beam's own
/// half length along the enemy's angle, which the real game writes hundreds of frames earlier: a launch
/// watched an enemy stand at `109.5,376.0` with `angle=2.011` and that anchor for seventeen seconds with
/// the behaviour nought, and the behaviour became 0x424730 on the very frame the player died at
/// `113.3,369.0`.
#[test]
fn the_bot_steps_out_of_a_beam_of_th08_before_it_fires() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-steps-out-before-a-beam-fires");
        game.image().enemies_alive(1);
        game.image().an_enemy_has_no_body(0);
        // Aimed down the player's own column with nothing in the behaviour: the anchor alone.
        game.image()
            .an_enemy_aims_a_beam(0, (MIDDLE, LOW), std::f32::consts::FRAC_PI_2);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        let pressed = game.image().the_word_the_keys_read() & PAD;
        assert_ne!(
            pressed & (LEFT | RIGHT),
            0,
            "the bot waited inside a beam that had not fired yet, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A boss of th08 is shot unfocused until the stage's norm is met, and focused after it**, where th07's is
/// shot focused on the same calm frame whatever — see `the_bot_playing_th07`'s
/// `the_bot_shoots_a_boss_slowed_down`: a hit of this game drops a time orb only with the human–youkai gauge
/// at its human end, which the unfocused shot leans it to, and the Last Spell is the time orbs and nothing
/// else. Once the count has reached the norm the orbs have bought it, and the fight wants the focused shot's
/// damage back.
#[test]
fn the_bot_shoots_a_boss_of_th08_unfocused_until_the_norm_is_met() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-shoots-a-boss-unfocused");
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().the_stage_has_its_time_orbs(1_200, 2_500);
        game.frames(4);
        let word = game.image().the_word_the_keys_read();
        assert_eq!(word & SHOT, SHOT, "the bot is not shooting the boss at all");
        assert_eq!(
            word & FOCUS,
            0,
            "the bot shot the boss focused under the norm with nothing to dodge:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The norm met: the same calm frame is one to hold the key on.
        game.image().the_stage_has_its_time_orbs(2_500, 2_500);
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_keys_read() & FOCUS,
            FOCUS,
            "the bot shot the boss unfocused with the norm already met:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And under `散符「真実の月（インビジブルフルムーン）」` the rules a recipe has dodge it `focused` hold the key
/// through the card**, the norm unmet or not and dodging or not: the one card of this game whose rings the
/// whole step was lost in, at the same place six attempts running with the key never held.
#[test]
fn the_rules_of_th08_dodge_invisible_full_moon_focused() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-dodge-focused",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: focused, spell: {INVISIBLE_FULL_MOON} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().the_stage_has_its_time_orbs(1_200, 2_500);
        game.image().a_card_is_up(INVISIBLE_FULL_MOON);
        // A calm frame under her with the norm unmet, which the rules otherwise shoot unfocused.
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_keys_read() & FOCUS,
            FOCUS,
            "the rules let the key up through a card dodged focused:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And a bullet coming down on the run, which they otherwise step out from under at the whole step.
        game.image().a_bullet_is_at(
            SLOTS[0],
            (MIDDLE, LOW - ABOVE),
            (0.0, A_FRESH_STEP),
            (4.0, 4.0),
        );
        game.frames(1);
        let word = game.image().the_word_the_keys_read();
        assert!(
            word & FOCUS == FOCUS && word & PAD != 0,
            "the rules did not step out from under it focused, pressing {word:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// The card, by its record's own number on Normal.
const INVISIBLE_FULL_MOON: usize = 113;

/// **And through a card's last 180 frames under a boss whose familiar is out, with the stage's norm unmet, the
/// rules hold their fire**, where with more of the card left, or the familiar gone, they shoot: a familiar
/// alive when the boss's attack ends is paid for, and past 180 frames left a card pays its floor whether it
/// is taken or runs out — see `Frame::familiars_are_worth_sparing`.
#[test]
fn the_rules_of_th08_spare_a_bosss_familiars_through_a_cards_last_frames() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-rules-spare-a-familiar");
        game.image().enemies_alive(2);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().an_enemy_is_a_familiar(1);
        game.image()
            .an_enemy_is_at(1, (MIDDLE + A_FAMILIAR_ASIDE, 100.0));
        game.image().the_stage_has_its_time_orbs(1_200, 2_500);
        game.image().a_card_is_up(A_CARD_WITH_FAMILIARS);
        let shooting_over = |frames: usize| {
            (0..frames)
                .filter(|_| {
                    game.frames(1);
                    game.image().the_word_the_keys_read() & SHOT != 0
                })
                .count()
        };
        // With most of the card left: the shot, as on every other frame of a fight.
        game.image().the_card_pays(0, A_CARDS_TIME, A_CARDS_TIME);
        assert_ne!(
            shooting_over(10),
            0,
            "the rules held their fire with most of a card left:\n  {}",
            game.log().lines().join("\n  "),
        );
        // In its last frames: no shot.
        game.image()
            .the_card_pays(0, IN_ITS_LAST_FRAMES, A_CARDS_TIME);
        assert_eq!(
            shooting_over(10),
            0,
            "the rules shot at a familiar through a card's last frames:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And with the familiar gone, the shot again.
        game.image().enemies_alive(1);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        assert_ne!(
            shooting_over(10),
            0,
            "the rules held their fire with no familiar out:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a card whose grazes hold the gauge short of its human end, with the stage's norm unmet, is played
/// focused for the rest of it once four hundred of its frames have gone by that way** — where one at that end is
/// played unfocused however long it has run, and the next card is played unfocused again: see
/// `Frame::at_the_gauges_human_end` and `steer`'s `HELD_OFF_THE_HUMAN_END_FOR`.
#[test]
fn the_rules_of_th08_play_the_rest_of_a_card_grazed_off_its_human_end_focused() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-a-card-grazed-off-its-human-end");
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().the_stage_has_its_time_orbs(1_200, 2_500);
        game.image().a_card_is_up(A_CARD_WITH_FAMILIARS);
        // **Counted from the frame after each change**, the word the keys read being the one the frame before
        // decided.
        let focused_over = |frames: usize| {
            game.frames(1);
            (0..frames)
                .filter(|_| {
                    game.frames(1);
                    game.image().the_word_the_keys_read() & FOCUS != 0
                })
                .count()
        };
        // At the human end, however long the card has run: unfocused.
        game.image().the_gauge_is_at(THE_HUMAN_END, THE_HUMAN_END);
        game.image().the_boss_has_attacked_for(0, 0);
        game.frames(10);
        game.image().the_boss_has_attacked_for(0, PAST_THE_HOLD);
        assert_eq!(
            focused_over(10),
            0,
            "the rules held the focus through a card at the gauge's human end:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Short of it, and not for long: unfocused still.
        game.image().a_card_is_up(A_CARD_WITH_FAMILIARS);
        game.image().the_boss_has_attacked_for(0, 0);
        game.image()
            .the_gauge_is_at(SHORT_OF_THE_HUMAN_END, THE_HUMAN_END);
        assert_eq!(
            focused_over(10),
            0,
            "the rules held the focus ten frames into a card short of the gauge's human end:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Four hundred of the card's frames short of it: the rest of the card focused.
        game.image().the_boss_has_attacked_for(0, PAST_THE_HOLD);
        assert_eq!(
            focused_over(10),
            10,
            "the rules did not play the rest of a card held short of the gauge's human end focused:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("and the rest of it is played focused"),
            "the card given over to the focused shot was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And the next card, its own clock back at nought: unfocused again.
        game.image().a_card_is_up(THE_NEXT_CARD);
        game.image().the_boss_has_attacked_for(0, 0);
        assert_eq!(
            focused_over(10),
            0,
            "the rules carried the focus into the next card:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The gauge's human end and a place short of it**, the first the line the game is laid out with and the
/// second the middle the three runs that never reached it through card 22 sat at; a card frame past four
/// hundred of them; and the card after card 22.
const THE_HUMAN_END: i16 = -9000;
const SHORT_OF_THE_HUMAN_END: i16 = -5000;
const PAST_THE_HOLD: i32 = 420;
const THE_NEXT_CARD: usize = 26;

/// How far beside the boss her familiar is put, which is off the player's own column; the card she is on,
/// stage 2's `22`; its time; and a time left inside its last 180 frames.
const A_FAMILIAR_ASIDE: f32 = 64.0;
const A_CARD_WITH_FAMILIARS: usize = 22;
const A_CARDS_TIME: i32 = 1_800;
const IN_ITS_LAST_FRAMES: i32 = 120;

/// **A recipe's `stand_at` is where the rules stand**, over every errand of their own: told an x at the
/// field's left edge, a run under a boss in the middle walks left rather than standing under it, and freed
/// of the place it walks back under the boss. Which is the one hand a recipe has for an attack the rules
/// cannot read — a beam laid down the middle of the field is dodged from its edge.
#[test]
fn the_rules_of_th08_stand_where_a_recipe_tells_them_to() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-stand-where-told",
            Work::wandering(WORK_US),
            |config| {
                // Told before the run: the step is carried out on the first frame the rules get.
                config.autoplay = Some(
                    "- stand_at: { x: 12 }\n- play_until: { frames: 40 }\n- stand_at: {}\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        // Told to stand at the left edge, with the boss overhead: the rules leave the boss for it.
        game.frames(2);
        assert_ne!(
            game.image().the_word_the_keys_read() & LEFT,
            0,
            "the rules stayed under the boss where a recipe told them to stand at the edge:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("tells the rules to stand at x = 12"),
            "the step did not say what it told the rules:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Freed once the frames are played, from the edge: back under the boss, which is to the right.
        game.frames_until("the rules freed of the place", 120, || {
            game.log()
                .said("frees the rules of the place they were told to stand")
        });
        game.frames(2);
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the rules did not go back under the boss once freed of the place:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a place told for one card is that card's alone**: the rules stay under the boss through another card,
/// leave for the told place the frame the card named comes up, and are back to their own errands the frame
/// it goes — with no step of the recipe run in between, which is what makes it no bracket's.
#[test]
fn the_rules_of_th08_stand_where_a_recipe_tells_them_through_the_card_it_names() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-stand-where-told-through-a-card",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- stand_at: { x: 12, spell: 94 }\n- play_until: { frames: 400 }\n".to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);

        // Another card up: the place is not its, so the rules stand under the boss.
        game.image().a_card_is_up(90);
        game.frames(2);
        assert_eq!(
            game.image().the_word_the_keys_read() & (LEFT | RIGHT),
            0,
            "the rules left the boss for a place told for another card:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The card named: the told place, and said.
        game.image().a_card_is_up(94);
        game.frames(2);
        assert_ne!(
            game.image().the_word_the_keys_read() & LEFT,
            0,
            "the rules stayed under the boss through the card a place was told for:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log()
                .said("card 94 is up, and the rules stand at x = 12 through it"),
            "the card's place taking over was not said:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the card gone, the run having got somewhere on its way to the edge: the rules' own errand again,
        // which is back under the boss to the right.
        game.frames(40);
        game.image().no_card_is_up();
        game.frames(2);
        assert!(
            game.log()
                .said("the rules stand where their own errands take them"),
            "the card's place lapsing was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the rules did not go back under the boss once the card was over:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way of dodging that is a place is waited at through the card it is named for**, each of them where
/// `orb_bot::steer` or `orb_bot::cards` holds its measurement: the field's middle at the low line, the middle
/// forty below the band `「幻視調律」` is born in, and either side of the middle at the low line.
#[test]
fn the_rules_of_th08_wait_where_the_way_a_card_is_dodged_puts_them() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-where-a-dodge-puts-them",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: in_the_middle, spell: 94 }\n\
                     - dodge: { with: visionary_tuning, spell: 105 }\n\
                     - dodge: { with: left_of_the_middle, spell: 120 }\n\
                     - dodge: { with: right_of_the_middle, spell: 128 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        for (card, name, at) in [
            (94, "in_the_middle", "192,368"),
            (105, "visionary_tuning", "192,290"),
            (120, "left_of_the_middle", "115,368"),
            (128, "right_of_the_middle", "288,368"),
        ] {
            game.image().a_card_is_up(card);
            game.frames(2);
            assert!(
                game.log()
                    .said(&format!("card {card} is up, and the rules dodge it {name}")),
                "card {card}'s way was not said:\n  {}",
                game.log().lines().join("\n  "),
            );
            assert!(
                game.log().said(&format!(
                    "wanted the place the card's own dodge puts it at {at}"
                )),
                "the rules did not want {at} through card {card}:\n  {}",
                game.log().lines().join("\n  "),
            );
        }
    });
}

/// **And a way of dodging that is a height is waited at at that height, the x the rules' own**: a third of the
/// way up from its floor of the rectangle the player is held in, `16` to `432` here, under the boss as the
/// rules' own errand puts them.
#[test]
fn the_rules_of_th08_wait_a_third_of_the_way_up_where_a_dodge_says_so() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-a-third-of-the-way-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: a_third_of_the_way_up, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(94);
        game.frames(2);
        assert!(
            game.log().said("a boss at 192,293"),
            "the rules did not want the boss's x a third of the way up:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way that weighs where the room will be walks to a place a stream will not cross**: a row of bullets
/// coming along the low line from the left wall at `3` a frame sweeps the run's own place inside a second, so
/// the place wanted is one at least two rows of the grid off that row.
#[test]
fn the_rules_of_th08_walk_out_of_the_way_of_a_stream_where_a_dodge_weighs_the_room_to_come() {
    in_its_own_process(|| {
        const A_STREAM: usize = 400;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-walk-where-there-will-be-room",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: where_there_will_be_room, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        for bullet in 0..16 {
            game.image().a_bullet_is_at(
                A_STREAM + bullet,
                (40.0 + 8.0 * bullet as f32, LOW),
                (3.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.image().a_card_is_up(94);
        game.frames(2);
        let wanted = game
            .log()
            .lines()
            .iter()
            .find_map(|line| {
                let (_, at) =
                    line.split_once("wanted the place the card's own dodge puts it at ")?;
                let (x, rest) = at.split_once(',')?;
                let y = rest.split(' ').next()?;
                Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
            })
            .unwrap_or_else(|| {
                panic!(
                    "the rules wanted no place of the way's:\n  {}",
                    game.log().lines().join("\n  ")
                )
            });
        assert!(
            (wanted.1 - LOW).abs() >= 48.0,
            "the rules wanted {wanted:?}, on the stream's row:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way that keeps off the edges does not walk into a wall to reach its place**: under a boss at the
/// left wall itself, `8`, the place wanted is at the wall, and a run `22` off it — where eight frames of any
/// step left, the focused one's included, land within `18` of the wall — stays where it is rather than step
/// left onto the wall with nothing at all to dodge.
#[test]
fn the_rules_of_th08_keep_off_a_wall_where_a_dodge_keeps_off_the_edges() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-keep-off-the-wall",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: under_the_boss, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((30.0, LOW));
        game.image().a_boss_is_fighting(0, (8.0, 100.0), 1000);
        game.image().a_card_is_up(94);
        game.frames(2);
        game.image().the_player_is_at((30.0, LOW));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & LEFT,
            0,
            "the rules stepped left onto the wall:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way dodged up and down the middle does not leave the middle to reach its place**: the run stood `10`
/// below the place it wants with a bullet standing `20` over it, so the taps up are hit and the diagonal taps up
/// come nearer that place than standing or a tap down — and the move taken is not across, every diagonal tap
/// landing more than `6` off the middle.
#[test]
fn the_rules_of_th08_keep_to_the_middle_where_a_dodge_keeps_to_it() {
    in_its_own_process(|| {
        const ABOVE: usize = 400;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-keep-to-the-middle",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: old_ties_bound_by_cedars, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, 300.0));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(94);
        game.frames(2);
        game.image().the_player_is_at((MIDDLE, 310.0));
        game.image()
            .a_bullet_is_at(ABOVE, (MIDDLE, 290.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & (LEFT | RIGHT),
            0,
            "the rules stepped across the middle:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way that dodges by tapping taps off fire aimed at the run, where a move held would run into
/// something**: a bullet coming at the run across the field from `30` left of it at `2` a frame, and one standing
/// `50` over it and one `50` under it. Held, the moves up and down reach those within the frames looked at and
/// the move right outruns the fire; tapped and let go, the move right is caught by it, and a tap up or down is
/// clear of all three and is the one taken.
#[test]
fn the_rules_of_th08_tap_off_fire_aimed_at_the_run_where_a_dodge_dodges_by_tapping() {
    in_its_own_process(|| {
        const ACROSS: usize = 400;
        const ABOVE: usize = 401;
        const UNDER: usize = 402;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-tap",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: old_ties_bound_by_cedars, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, 300.0));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(94);
        game.frames(2);
        game.image().the_player_is_at((MIDDLE, 300.0));
        game.image()
            .a_bullet_is_at(ACROSS, (MIDDLE - 30.0, 300.0), (2.0, 0.0), (4.0, 4.0));
        game.image()
            .a_bullet_is_at(ABOVE, (MIDDLE, 250.0), (0.0, 0.0), (4.0, 4.0));
        game.image()
            .a_bullet_is_at(UNDER, (MIDDLE, 350.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let word = game.image().the_word_the_keys_read();
        assert_eq!(
            word & (LEFT | RIGHT),
            0,
            "the rules went across the fire's way:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            word & (UP | DOWN),
            0,
            "the rules did not tap off the fire:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way that keeps away from where a fall of fire lands waits across the middle from it**: bullets
/// falling through the lowest quarter of the field from `232` to `272` across land right of the middle, `192`, so
/// the place wanted is `96` left of it at the low line — and bullets higher up or going up are no part of the
/// fall, though they are left of the middle.
#[test]
fn the_rules_of_th08_wait_across_the_middle_from_where_a_fall_lands() {
    in_its_own_process(|| {
        const A_FALL: usize = 400;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-keep-away-from-where-the-fall-lands",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: away_from_where_the_fall_lands, spell: 94 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        for bullet in 0..6 {
            game.image().a_bullet_is_at(
                A_FALL + bullet,
                (232.0 + 8.0 * bullet as f32, 360.0),
                (0.5, 2.0),
                (4.0, 4.0),
            );
        }
        // **And none of these**: four high up, four going up, all of them left of the middle.
        for bullet in 6..10 {
            game.image().a_bullet_is_at(
                A_FALL + bullet,
                (40.0, 120.0 + 8.0 * bullet as f32),
                (0.0, 2.0),
                (4.0, 4.0),
            );
        }
        for bullet in 10..14 {
            game.image().a_bullet_is_at(
                A_FALL + bullet,
                (40.0, 300.0 + 8.0 * bullet as f32),
                (0.0, -2.0),
                (4.0, 4.0),
            );
        }
        game.image().a_card_is_up(94);
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 96,368"),
            "the rules did not want the other side of the middle:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a way named for the fire after a card is taken up once that card has ended with the stage's boss still
/// up**, and put down as the next card starts — the fire between two cards having no number of its own to be
/// named by. **Cards 95 and 96**, which the autoplay engine names no way for: its own for card 94 is the middle as
/// well, and a way the same as the one before it is no way taken up.
#[test]
fn the_rules_of_th08_dodge_the_fire_after_a_card_the_way_named_for_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-dodge-the-fire-after-a-card",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: in_the_middle, after_spell: 95 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().the_stream_reads_the_track(THE_FIGHTS_TRACK);
        game.image().a_card_is_up(95);
        game.frames(2);
        assert!(
            !game.log().said("the fire after card 95 is up"),
            "the way was taken up while card 95 ran:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.image().no_card_is_up();
        game.frames(2);
        assert!(
            game.log()
                .said("the fire after card 95 is up, and the rules dodge it in_the_middle"),
            "the way was not taken up after card 95:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 192,368"),
            "the rules did not want the middle after card 95:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.image().a_card_is_up(96);
        game.frames(2);
        assert!(
            game.log()
                .said("the rules dodge the way their own rules do"),
            "the way was not put down as card 96 started:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And that place moved on a retry of the chapter the card is in**, by as much as the rules said they would
/// wait aside, the way the boss's own x and the middle are: held still, a retry is the attempt before it over
/// again — one Extra was hit at `198.3,428.0` attempt after attempt with the middle a card's way holds held
/// through each.
///
/// **Walked into rather than placed**, since a chapter is taken off the stage's build — see
/// `a_pointdevice_run_of_th08` — and the death laid out on the frames before the stage's own first bullet, so
/// that nothing but the card is in the field.
#[test]
fn the_rules_of_th08_wait_aside_of_the_place_a_cards_way_holds_on_a_retry() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-aside-on-a-retry",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "{WALKED_INTO_A_RUN}\
                     - dodge: {{ with: in_the_middle, spell: 94 }}\n\
                     - play_until: {{ frames: 6000 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.stages_last(100_000);
        game.plays_its_own_stage();
        game.frames_until("the run", 6000, || game.state().playing);
        game.frames_until(
            "the stage's first chapter and the way handed over",
            600,
            || {
                log.said("stage 1 chapter 1 (stage start)")
                    && log.said("has the rules dodge card 94")
            },
        );

        let the_card_is_up = || {
            game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
            game.image().a_card_is_up(94);
        };
        the_card_is_up();
        game.frames(8);
        assert!(
            log.said("wanted the place the card's own dodge puts it at 192,368"),
            "the first attempt did not wait at the middle:\n  {}",
            log.lines().join("\n  "),
        );

        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.frames(8);
        assert!(
            log.said("died in chapter 1"),
            "the death was not seen:\n  {}",
            log.lines().join("\n  "),
        );
        game.frames_until("the chapter taken again", 240, || {
            log.said("retry: the chapter again")
        });

        // What the chapter put back is a stage with no card up in it, so the card again.
        let from = log.written();
        the_card_is_up();
        game.frames_until("the retry's aside said", 8, || {
            log.said("retry 1 of this chapter, and the rules wait ")
        });
        let aside: f32 = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("retry 1 of this chapter, and the rules wait ")?;
                after.split_once(" to the side")?.0.parse().ok()
            })
            .expect("the retry said how far aside the rules wait");
        assert_ne!(aside, 0.0, "the retry waits where the first attempt did");
        let wanted = format!(
            "wanted the place the card's own dodge puts it at {:.0},368",
            MIDDLE + aside
        );
        game.frames_until("the retry waiting aside of the middle", 8, || {
            log.said_since(from, &wanted)
        });
    });
}

/// **And a way that stands under the boss moved aside of her on a retry** the same: held under her, the nonspell
/// after th10's card 78 was retried six times on a stage practice and hit at `-177.6,432.0` 879 frames in on every
/// one of them.
#[test]
fn the_rules_of_th08_wait_aside_of_the_boss_a_cards_way_stands_under_on_a_retry() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-aside-of-the-boss-on-a-retry",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "{WALKED_INTO_A_RUN}\
                     - dodge: {{ with: under_the_boss, spell: 94 }}\n\
                     - play_until: {{ frames: 6000 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.stages_last(100_000);
        game.plays_its_own_stage();
        game.frames_until("the run", 6000, || game.state().playing);
        game.frames_until(
            "the stage's first chapter and the way handed over",
            600,
            || {
                log.said("stage 1 chapter 1 (stage start)")
                    && log.said("has the rules dodge card 94")
            },
        );

        let boss_x = MIDDLE + 40.0;
        let the_card_is_up = || {
            game.image().a_boss_is_fighting(0, (boss_x, 100.0), 1000);
            game.image().a_card_is_up(94);
        };
        the_card_is_up();
        game.frames(8);
        let under_her = format!("wanted the place the card's own dodge puts it at {boss_x:.0},368");
        assert!(
            log.said(&under_her),
            "the first attempt did not wait under the boss:\n  {}",
            log.lines().join("\n  "),
        );

        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.frames(8);
        assert!(
            log.said("died in chapter 1"),
            "the death was not seen:\n  {}",
            log.lines().join("\n  "),
        );
        game.frames_until("the chapter taken again", 240, || {
            log.said("retry: the chapter again")
        });

        let from = log.written();
        the_card_is_up();
        game.frames_until("the retry's aside said", 8, || {
            log.said("retry 1 of this chapter, and the rules wait ")
        });
        let aside: f32 = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("retry 1 of this chapter, and the rules wait ")?;
                after.split_once(" to the side")?.0.parse().ok()
            })
            .expect("the retry said how far aside the rules wait");
        assert_ne!(aside, 0.0, "the retry waits where the first attempt did");
        let wanted = format!(
            "wanted the place the card's own dodge puts it at {:.0},368",
            boss_x + aside
        );
        game.frames_until("the retry waiting aside of the boss", 8, || {
            log.said_since(from, &wanted)
        });
    });
}

/// **And the place a way that finds its own answers moved on a retry** the same — the way across the middle from
/// where a fall of fire lands, which with nothing falling is the middle: held still, th10's stage 4 waterfall was
/// retried six times from its chapter and hit at `184.0,432.0` on the same frame, stage frame 4,874, every one of them.
#[test]
fn the_rules_of_th08_wait_aside_of_the_place_a_stretchs_way_finds_on_a_retry() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-aside-of-a-stretchs-place-on-a-retry",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "{WALKED_INTO_A_RUN}\
                     - dodge: {{ with: away_from_where_the_fall_lands, stage: 1, from: 0, to: 60000 }}\n\
                     - play_until: {{ frames: 6000 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.stages_last(100_000);
        game.plays_its_own_stage();
        game.frames_until("the run", 6000, || game.state().playing);
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        let a_boss_is_up = || game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        a_boss_is_up();
        game.frames(8);
        assert!(
            log.said(&format!(
                "wanted the place the card's own dodge puts it at {MIDDLE:.0},368"
            )),
            "the first attempt did not wait at the middle:\n  {}",
            log.lines().join("\n  "),
        );

        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.frames(8);
        assert!(
            log.said("died in chapter 1"),
            "the death was not seen:\n  {}",
            log.lines().join("\n  "),
        );
        game.frames_until("the chapter taken again", 240, || {
            log.said("retry: the chapter again")
        });

        let from = log.written();
        a_boss_is_up();
        game.frames_until("the retry's aside said", 8, || {
            log.said("retry 1 of this chapter, and the rules wait ")
        });
        let aside: f32 = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("retry 1 of this chapter, and the rules wait ")?;
                after.split_once(" to the side")?.0.parse().ok()
            })
            .expect("the retry said how far aside the rules wait");
        assert_ne!(aside, 0.0, "the retry waits where the first attempt did");
        let wanted = format!(
            "wanted the place the card's own dodge puts it at {:.0},368",
            MIDDLE + aside
        );
        game.frames_until("the retry waiting aside of the middle", 8, || {
            log.said_since(from, &wanted)
        });
    });
}

/// **The walk from the title into a Normal run in Pointdevice mode**, by this game's own screen numbers — the
/// same steps `a_death_answered_by_the_chapter_again` walks in by.
const WALKED_INTO_A_RUN: &str = "\
- wait_until: { screen: 0 }
  give_up_after: 1800
- press: [up]
  frames: 30
  until: { cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 4 }
  give_up_after: 1800
- press: [decide]
  frames: 30
  until: { screen: 5 }
  give_up_after: 900
- press: [decide]
  frames: 12
- wait_until: { frames: 120 }
- press: [decide]
  frames: 12
";

/// **A `play_until` naming a chapter of the next stage waits out that stage's build**, and ends at the
/// chapter it names. The chapters are the stage before's until the game is in the next one, and this game's
/// stage word is the next one's for the whole build — so a step waiting for stage 2's second chapter read the
/// stage as 2 and the chapter as stage 1's last, and was over before stage 2 began.
#[test]
fn a_play_until_of_th08_waits_out_the_build_of_the_stage_it_names() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-play-until-waits-out-a-build",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay =
                    Some("- play_until: { stage: 2, chapter: 2 }\n  give_up_after: 100000\n- stand_at: { x: 12 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        // The title's own coming-up spent first, a run of hundreds of frames being one the laid-out game
        // would otherwise pull back to its title part way through.
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        // **Stage 1 built on the loader and played by the game's own clock**, a run put in place going
        // through no build and its clock standing: the chapters take a stage as begun off the loader's word
        // clearing — see `Th08::a_stage_has_just_been_built` — and as settled off the frames after.
        game.stages_last(100_000);
        game.plays_its_own_stage();
        game.moves_on_to_the_stage(FIRST_STAGE);
        game.frames_until("stage 1 built", 600, || {
            !game.image().the_loader_is_running()
        });
        // Stage 1 taken to its second chapter, a boss's own.
        game.frames_until("stage 1's start", 600, || {
            game.log().said("stage 1 chapter 1 (stage start)")
        });
        // A minute of frames on, a boss arriving sooner being the same chapter renamed.
        game.frames(120);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.frames_until("stage 1's second chapter", 600, || {
            game.log().said("stage 1 chapter 2")
        });
        game.image().no_boss_is_fighting();
        // **The stage left mid-dialogue**, which is how a stage of the real game ends — the boss's last
        // words, and the switch inside them — and a line still up is a frame the chapters read nothing
        // off, so the stage before's number is what they hold when the build starts.
        game.image().a_line_of_dialogue_is_up(1);
        game.frames(2);
        // Stage 2 built on the loader, and its own start reached, the step still waiting through both.
        game.moves_on_to_the_stage(FIRST_STAGE + 1);
        game.frames_until("stage 2 built", 600, || {
            !game.image().the_loader_is_running()
        });
        game.frames_until("stage 2's start", 600, || {
            game.log().said("stage 2 chapter 1 (stage start)")
        });
        assert!(
            !game.log().said("tells the rules to stand"),
            "the step waiting for stage 2's second chapter ended before stage 2 began:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And over at the chapter it named.
        game.frames(120);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.frames_until("the step over at stage 2's second chapter", 600, || {
            game.log().said("tells the rules to stand at x = 12")
        });
    });
}

/// **A recipe can tell the rules a height and leave where along it to them**, which is what an attack whose
/// ground is a y wants: told 240 under a boss, the rules climb out of the band they otherwise live in and
/// the x stays the errand's.
#[test]
fn the_rules_of_th08_stand_at_the_height_a_recipe_tells_them() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-stand-at-a-height",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- stand_at: { y: 240 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.frames(2);
        assert!(
            game.log()
                .said("tells the rules to stand where they were going, at y = 240"),
            "the step did not say what it told the rules:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            game.image().the_word_the_keys_read() & UP,
            0,
            "the rules stayed low where a recipe told them a height:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A recipe can tell the rules a distance to keep from the boss**, and the rules keep it on the side of
/// her they are on: told 120 with the boss at the middle and the player a little to her left, the rules go
/// left, and once the boss has moved to the left edge — where 120 fits only on her right — they cross to her
/// right.
#[test]
fn the_rules_of_th08_keep_the_distance_from_the_boss_a_recipe_tells_them() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-keep-away-from-the-boss",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- stand_at: { away_from_the_boss: 120 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE - 10.0, LOW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.frames(2);
        assert!(
            game.log()
                .said("tells the rules to stand 120 away from the boss"),
            "the step did not say what it told the rules:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            game.image().the_word_the_keys_read() & LEFT,
            0,
            "the rules did not move away from the boss on their own side of her:\n  {}",
            game.log().lines().join("\n  "),
        );
        // The boss at the left edge with the player just right of her: 120 fits on her right alone.
        game.image().a_boss_is_fighting(0, (20.0, 100.0), 1000);
        game.image().the_player_is_at((40.0, LOW));
        game.frames(2);
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the rules did not go to the one side of the boss the distance fits on:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under a boss and over a field with something in it, an item worth nothing is picked up only where it
/// costs nothing.** Class 7 is what an erased bullet leaves, three at a time, and a spell card is where they
/// fall thickest: across such a field it is no errand, and a step away it is taken on the way.
///
/// **A 1up below the low line is followed down**, where any other item there is waited above: one a boss drops
/// onto the player is under it as often as over it, and a run that stood at the low line over one watched it
/// fall out of the field — a Normal clear wanted `a 1up at 192,368` from `191,368` for 270 frames of stage 3
/// and never took it.
#[test]
fn the_bot_of_th08_follows_a_1up_down_past_the_low_line() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-follows-a-1up-down");
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        // A power item there first: waited above, at the low line the player already stands on.
        game.image().an_item_is_at((MIDDLE, BELOW_THE_LOW_LINE));
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & DOWN,
            0,
            "the bot went below the low line for a power item:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And a 1up in its place: gone down for.
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().an_item_is_at((MIDDLE, BELOW_THE_LOW_LINE));
        game.image().the_item_is_of_class(A_1UP);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & DOWN,
            0,
            "the bot stood above a 1up below the low line:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where that test puts the item**, halfway from the low line to the floor the player is held above, and
/// the class this game gives a 1up — see `sight::worth`.
const BELOW_THE_LOW_LINE: f32 = LOW + 32.0;
const A_1UP: u8 = 5;

/// **The field with something in it is the whole of the reason**, which is why this test puts bullets on it:
/// what the rule was written off is a run that wanted `an item of no place of its own at 335,384` from
/// `234,387` **under 390 bullets** and died crossing for it thirteen times over. A field with nothing inside
/// `steer::CLEAR_ENOUGH_TO_SWEEP` of the player is not that frame, and the case below holds what the rules do
/// there — on this game those items are its **time orbs**, the count its Last Spells are due on.
#[test]
fn the_bot_crosses_th08s_field_for_no_item_worth_nothing() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-leaves-a-scrap-alone");
        // A boss up, over the middle: what the rules want with nothing to collect is its own x, low, which
        // is where the player already stands.
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        // And a bullet of its own on the item's own side, which is what makes the field one to stay on: near
        // enough that the field is not clear, and across the field from the item so that the one direction
        // this case is about is the one both the fire and the item are in.
        game.image().a_bullet_is_at(
            SLOTS[0],
            (MIDDLE + A_BULLET_BESIDE_THE_PLAYER, LOW),
            (0.0, A_SLOW_STEP),
            (4.0, 4.0),
        );
        // Across the field: not crossed for, the rules staying on their own side of it.
        game.image().an_item_is_at((MIDDLE + 100.0, LOW));
        game.image().the_item_is_of_class(A_SCRAP);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the bot crossed a field with something on it for a scrap worth nothing:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And with that bullet gone, a step away: taken.
        game.image().the_bullet_is_gone(SLOTS[0]);
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().an_item_is_at((MIDDLE + 6.0, LOW));
        game.image().the_item_is_of_class(A_SCRAP);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the bot left a scrap a step away where it stood:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And a power item across the field is still gone for, which is the errand this leaves alone.
        game.image().an_item_is_at((MIDDLE + 100.0, LOW));
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the bot left a power item across the field:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And over a field a captured card has cleared it is left alone as well, which was measured and not
/// assumed.** Those items are this game's **time orbs** — a card taken here cancels every bullet on the
/// screen and leaves one per bullet cancelled — and the count a stage's Last Spell is due on is counted in
/// them, which is what the route to this game's own last stage turns on. One clear's stage 1 read **2,094 of
/// the 2,500** its norm asks with the count climbing in steps of 21 to 25 and no card payout in any of them,
/// so the orbs looked like the half to buy: told to cross for one wherever nothing was inside 96 units, the
/// same stage read **1,872**, and told the field's middle through that boss's whole fight it read **1,977** —
/// both below the run told nothing. What the crossing costs is the damage not done while the shot is off the
/// boss, and the payout is what the norm is really short of.
#[test]
fn the_bot_of_th08_leaves_a_time_orb_on_a_cleared_field_too() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-bot-leaves-a-cleared-field-alone");
        // The same boss up and the same item across the field — and nothing else on it at all, which is the
        // screen a card has just been taken on.
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().an_item_is_at((MIDDLE + 100.0, LOW));
        game.image().the_item_is_of_class(A_SCRAP);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the bot crossed a cleared field for a time orb, which two launches say costs more than it \
             pays:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under a ring the boss fires from overhead, the rules wait in the middle and not at an edge** — see
/// `steer`'s `IN_THE_MIDDLE_UNDER_HER`. A ring counted as it stands reads the middle of the band as a
/// pile, its bullets coming down thicker under her than out at the sides, and the rules walked to an edge of
/// the field under every one of them, where the next ring closes the corner.
///
/// The ring here counts `0, 15, 25, 15, 0` across the five boxes at the old tenth and twentieth, which is the
/// middle badly fuller than the edge and every box between piled: the emptiest box to wait in was the edge.
#[test]
fn the_rules_of_th08_wait_in_the_middle_under_a_ring_the_boss_fires_from_overhead() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-rules-wait-under-a-ring");
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 1000);
        // **Going out a step a frame, the way the game's own update moves it**: a ring that stood where its
        // steps say it is leaving reads as bullets a script is sweeping, which is another rule's to answer.
        for frame in 0..2 {
            let radius = A_RINGS_RADIUS + A_RINGS_SPEED * frame as f32;
            for bullet in 0..A_RINGS_BULLETS {
                let angle = std::f32::consts::TAU * bullet as f32 / A_RINGS_BULLETS as f32;
                let (across, down) = (angle.cos(), angle.sin());
                game.image().a_bullet_is_at(
                    A_RINGS_FIRST_SLOT + bullet,
                    (MIDDLE + radius * across, A_RINGS_CENTRE + radius * down),
                    (A_RINGS_SPEED * across, A_RINGS_SPEED * down),
                    (8.0, 8.0),
                );
            }
            game.image().its_bullets_are_a_frame_older();
            game.frames(1);
        }
        let log = game.log();
        assert!(
            log.said("autoplay: wanted a boss at 192") && !log.said("wanted the emptiest box"),
            "the rules left the middle under a ring from overhead:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And under the fire of `禁薬「蓬莱の薬」`, which is no ring, the rules stay under the boss as well.** The fire
/// laid out is one frame of it as a Normal clear counted it — `7, 81, 212, 85, 7` across the five boxes, the
/// middle's own before its discount — and Eirin at the top of the middle: the frame that run wanted the
/// emptiest box at `x = 56` on, with the two boxes beside the middle full and the edges nearly empty. The
/// card's fire leaves her in every direction and at every speed at once, so what the rules read of it was
/// never a ring and the edge was where they went; there they stood for 3,600 of the card's 7,592 frames,
/// which is where the next wave closes a corner.
#[test]
fn the_rules_of_th08_stay_under_a_boss_overhead_whose_fire_is_no_ring() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-rules-stay-under-the-boss");
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 1000);
        // **Falling a step a frame, the way the game's own update moves them**, for the reason the ring's
        // own test gives.
        for frame in 0..2 {
            let mut slot = A_RINGS_FIRST_SLOT;
            for (box_, bullets) in HOURAI_ELIXIRS_FIRE.iter().enumerate() {
                for bullet in 0..*bullets {
                    let across = A_BOXS_LEFT
                        + A_BOX * box_ as f32
                        + A_BOX * (bullet as f32 + 0.5) / *bullets as f32;
                    let down = A_FALLING_BULLETS_TOP
                        + 13.0 * (bullet % 10) as f32
                        + A_FALLING_STEP * frame as f32;
                    game.image().a_bullet_is_at(
                        slot,
                        (across, down),
                        (0.0, A_FALLING_STEP),
                        (8.0, 8.0),
                    );
                    slot += 1;
                }
            }
            game.image().its_bullets_are_a_frame_older();
            game.frames(1);
        }
        let log = game.log();
        let wanted: Vec<f32> = log
            .lines()
            .iter()
            .filter_map(|line| line.split("autoplay: wanted ").nth(1))
            .filter_map(|it| it.split(" at ").nth(1))
            .filter_map(|it| it.split(',').next())
            .filter_map(|x| x.parse().ok())
            .collect();
        assert!(
            !wanted.is_empty() && wanted.iter().all(|x| (x - MIDDLE).abs() < A_BOX / 2.0),
            "the rules left the middle under a boss overhead, wanting {wanted:?}:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And out at an edge under a boss overhead, the rules go back for the middle and not the box beside them.**
/// The fire laid out is `散符「真実の月（インビジブルフルムーン）」` as a no-bomb clear counted it on average,
/// `70, 80, 105, 82, 66` before the discount under her, with the run at the right edge where that clear stood
/// on 2,284 of the card's 3,060 frames: the middle is the emptiest box once counted, and what the rules wanted
/// there was the box beside the edge — the nearest to them not piled — while each ring going out past them
/// took them back to it.
#[test]
fn the_rules_of_th08_at_an_edge_under_a_boss_overhead_go_back_for_the_middle() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-rules-go-back-for-the-middle");
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 1000);
        for frame in 0..2 {
            let mut slot = A_RINGS_FIRST_SLOT;
            for (box_, bullets) in INVISIBLE_FULL_MOONS_FIRE.iter().enumerate() {
                for bullet in 0..*bullets {
                    let across = A_BOXS_LEFT
                        + A_BOX * box_ as f32
                        + A_BOX * (bullet as f32 + 0.5) / *bullets as f32;
                    let down = A_FALLING_BULLETS_TOP
                        + 13.0 * (bullet % 10) as f32
                        + A_FALLING_STEP * frame as f32;
                    game.image().a_bullet_is_at(
                        slot,
                        (across, down),
                        (0.0, A_FALLING_STEP),
                        (8.0, 8.0),
                    );
                    slot += 1;
                }
            }
            game.image().its_bullets_are_a_frame_older();
            game.image().the_player_is_at((AT_THE_RIGHT_EDGE, LOW));
            game.frames(1);
        }
        let log = game.log();
        assert!(
            log.said("autoplay: wanted the emptiest box at 192,"),
            "the rules at an edge under a boss overhead wanted somewhere other than the middle:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And with the boss beside an edge and not overhead, the rules leave the box they are in for the middle and
/// not for the edge.** The fire laid out is card 152 as a Kaguya clear counted it 157 frames before a hit at
/// `44.6,369.5` — `0, 6, 0, 4, 2` across the five boxes, with the run and the boss in the box beside the left
/// edge: the edge and the middle read equally empty and were equally near, the edge came first, and five
/// clears were hit at that wall 355 frames into the card.
#[test]
fn the_rules_of_th08_beside_an_edge_leave_for_the_middle_and_not_the_edge() {
    in_its_own_process(|| {
        let game = in_a_stage("th08-the-rules-leave-for-the-middle-beside-an-edge");
        game.image()
            .a_boss_is_fighting(0, (BESIDE_THE_LEFT_EDGE, A_RINGS_CENTRE), 1000);
        for frame in 0..2 {
            let mut slot = A_RINGS_FIRST_SLOT;
            for (box_, bullets) in CARD_152S_FIRE.iter().enumerate() {
                for bullet in 0..*bullets {
                    let across = A_BOXS_LEFT
                        + A_BOX * box_ as f32
                        + A_BOX * (bullet as f32 + 0.5) / *bullets as f32;
                    let down = A_FALLING_BULLETS_TOP
                        + 13.0 * (bullet % 10) as f32
                        + A_FALLING_STEP * frame as f32;
                    game.image().a_bullet_is_at(
                        slot,
                        (across, down),
                        (0.0, A_FALLING_STEP),
                        (8.0, 8.0),
                    );
                    slot += 1;
                }
            }
            game.image().its_bullets_are_a_frame_older();
            game.image().the_player_is_at((BESIDE_THE_LEFT_EDGE, LOW));
            game.frames(1);
        }
        let log = game.log();
        assert!(
            log.said("autoplay: wanted the emptiest box at 192,")
                && !log.said("autoplay: wanted the emptiest box at 56,"),
            "the rules beside an edge left for somewhere other than the middle:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And under `滅罪「正直者の死」` dodged `behind_the_laser`, the rules wait in the middle between its lasers,
/// whatever the boxes say**: the lasers come from either side by turns and the run steps to the side each comes
/// from, which at a wall is no room at all. The fire laid out is thickest in the middle and the run is in the box
/// right of it, which the boxes alone would keep it in or send it under the boss for; one Extra's boxes sent it to
/// the left wall, and it was hit at `x = 8` five times.
#[test]
fn the_rules_of_th08_wait_in_the_middle_between_the_lasers_of_honest_mans_death() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-wait-in-the-middle-through-honest-mans-death",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: behind_the_laser, spell: {HONEST_MANS_DEATH} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 1000);
        game.image().a_card_is_up(HONEST_MANS_DEATH);
        for frame in 0..3 {
            let mut slot = A_RINGS_FIRST_SLOT;
            for (box_, bullets) in HONEST_MANS_DEATHS_FIRE.iter().enumerate() {
                for bullet in 0..*bullets {
                    let across = A_BOXS_LEFT
                        + A_BOX * box_ as f32
                        + A_BOX * (bullet as f32 + 0.5) / *bullets as f32;
                    let down = A_FALLING_BULLETS_TOP
                        + 13.0 * (bullet % 10) as f32
                        + A_FALLING_STEP * frame as f32;
                    game.image().a_bullet_is_at(
                        slot,
                        (across, down),
                        (0.0, A_FALLING_STEP),
                        (8.0, 8.0),
                    );
                    slot += 1;
                }
            }
            game.image().its_bullets_are_a_frame_older();
            game.image().the_player_is_at((RIGHT_OF_THE_MIDDLE, LOW));
            game.frames(1);
        }
        // The last frame's wanting, the card's own number having been read on the first.
        let log = game.log();
        let lines = log.lines();
        let last = lines
            .iter()
            .rev()
            .find(|line| line.contains("autoplay: wanted "));
        assert!(
            last.is_some_and(|line| {
                line.contains("autoplay: wanted the place the card's own dodge puts it at 192,368")
            }),
            "the rules did not wait in the middle between the card's lasers:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **The fire that test lays out**, `60, 84, 200, 100, 60` over four — the middle thickest as it stands, which
/// is that card's own — and where the run is put: the middle of the box right of the middle's.
const HONEST_MANS_DEATHS_FIRE: [usize; 5] = [15, 21, 50, 25, 15];
const RIGHT_OF_THE_MIDDLE: f32 = 266.0;

/// **The fire that test lays out**, `70, 80, 105, 82, 66` over four, and where the run is put: the right edge's
/// own box, where the rules keep `steer`'s `IN_FROM_THE_SIDE` off the wall.
const INVISIBLE_FULL_MOONS_FIRE: [usize; 5] = [17, 20, 26, 21, 16];
const AT_THE_RIGHT_EDGE: f32 = 330.0;

/// **The fire that test lays out**, as bullets of the size whose one box counts it four: `7, 81, 212, 85, 7`
/// over four is two, twenty, fifty-three, twenty-one and two — and falling from the top of the band the boxes
/// are counted over, which is sixty below the field's middle on a field 416 deep.
const HOURAI_ELIXIRS_FIRE: [usize; 5] = [2, 20, 53, 21, 2];

/// **And the fire the card 152 test lays out** in the same bullets: `0, 6, 0, 4, 2` over four, rounded so that
/// the box beside the left edge is piled and nothing else is — two bullets there and one in each of the two boxes
/// right of the middle — with the boss and the run both in the box beside the left edge, where that clear
/// stood.
const CARD_152S_FIRE: [usize; 5] = [0, 2, 0, 1, 1];
const BESIDE_THE_LEFT_EDGE: f32 = 135.0;
const A_BOXS_LEFT: f32 = 8.0;
const A_BOX: f32 = 368.0 / 5.0;
const A_FALLING_BULLETS_TOP: f32 = 200.0;
const A_FALLING_STEP: f32 = 2.0;

/// **The ring that test lays out**: forty-eight bullets out of a boss at the middle of the top, eighty out
/// from her and two a frame — this file's own numbers, what they have to be being a ring whose fire comes into
/// the band thickest under her.
const A_RINGS_BULLETS: usize = 48;
const A_RINGS_FIRST_SLOT: usize = 400;
const A_RINGS_CENTRE: f32 = 100.0;
const A_RINGS_RADIUS: f32 = 80.0;
const A_RINGS_SPEED: f32 = 2.0;

/// **Under `滅罪「正直者の死」` the rules a recipe has dodge it `behind_the_laser` step the short way towards
/// the side its laser comes from, and are behind it for as long as it sweeps on once live.** The laser is the one
/// the card draws, read off the real game every frame of three attempts: out of Mokou at `192,128`, drawn
/// `60` frames before it goes live and turning `π/400` a frame onto where the player stood when it appeared
/// — here from the player's right, so its side is the right.
///
/// **And it takes crossing ground the laser is drawn over**, which is what the rules would not do: a warning
/// judged where it is drawn on each frame before it fires shut the step behind it, and the moves running
/// ahead of it — to the left, away from the laser drawn at them — were the ones clear. Which is how the
/// card's name has it: the honest dodge is the one that dies.
#[test]
fn the_rules_of_th08_step_behind_the_laser_of_honest_mans_death_before_it_goes_live() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-step-behind-the-laser",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: behind_the_laser, spell: {HONEST_MANS_DEATH} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image().a_boss_is_fighting(0, MOKOU, 1000);
        game.image().a_card_is_up(HONEST_MANS_DEATH);
        let onto = (LOW - MOKOU.1).atan2(MIDDLE - MOKOU.0);
        let first = onto - ITS_TURN * ITS_WARNING as f32;
        game.image()
            .a_laser_is_at(0, MOKOU, first, (0.0, ITS_REACH), ITS_WIDTH);
        game.image().the_laser_fires_in(0, ITS_WARNING);
        game.image().the_laser_was_turned_to(0, first);
        for frame in 1..=ITS_WARNING + LIVE_FOR {
            game.frames(1);
            if frame == ITS_WARNING {
                game.image().the_laser_goes_live(0);
            }
            let angle = first + ITS_TURN * frame as f32;
            game.image().the_laser_was_turned_to(0, angle);
            if frame < ITS_WARNING {
                continue;
            }
            // Across the lane, positive on the side it is turning onto: behind it is the negative side.
            let (x, y) = game.image().where_the_player_is();
            let across = -(x - MOKOU.0) * angle.sin() + (y - MOKOU.1) * angle.cos();
            assert!(
                across < -ITS_WIDTH / 4.0,
                "the rules were not behind the laser {} frame(s) after it went live, {across:.1} across it \
                 at {x:.1},{y:.1}:\n  {}",
                frame - ITS_WARNING,
                game.log().lines().join("\n  "),
            );
        }
        assert!(
            game.log().said(&format!(
                "card {HONEST_MANS_DEATH} is up, and the rules dodge it behind_the_laser"
            )),
            "the card's own way of dodging was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under `魔砲「ファイナルスパーク」` the rules a recipe has dodge it `out_of_the_following_laser` keep out of
/// its beam at the width it fires at while it only warns, and move away from it as it turns after them.** The
/// beam is the one the card fires, read off the real game's enemies every frame: out of Marisa at `161,103`,
/// aimed where the player stood and held there for `118` frames, then `240` wide and turning `0.0036` rad a
/// frame onto the player until it points at them. A run `83.7` off its middle — clear of the `160` a beam
/// only warning is read at — was hit the frame it fired, and stood still while it turned.
#[test]
fn the_rules_of_th08_keep_out_of_the_following_laser_of_final_spark() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-keep-out-of-the-following-laser",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: out_of_the_following_laser, spell: {FINAL_SPARK} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        // Her and the enemies the beam is laid by, the one before it with no body and no beam either.
        game.image().enemies_alive(ITS_BEAMS_SLOT + 1);
        game.image().a_boss_is_fighting(0, MARISA, 1000);
        game.image().an_enemy_has_no_body(ITS_BEAMS_SLOT - 1);
        game.image().an_enemy_has_no_body(ITS_BEAMS_SLOT);
        game.image().a_card_is_up(FINAL_SPARK);
        // Warning, straight down from her, with the run where the run was: out of the lane it will fire in,
        // which is to the right, away from her column and towards the field's own middle.
        let down = std::f32::consts::FRAC_PI_2;
        game.image()
            .the_player_is_at((MARISA.0 + WHERE_THE_RUN_WAS, LOW));
        game.image()
            .an_enemy_aims_a_beam(ITS_BEAMS_SLOT, beams_middle(down), down);
        // Two frames, the card's own number being read on the first.
        game.frames(2);
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the rules stood {WHERE_THE_RUN_WAS} off a following laser's warning:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Firing, and turning onto a run clear of it: away from it, and not towards her.
        let (x, y) = FIRING_ONTO;
        let (dx, dy) = (x - MARISA.0, y - MARISA.1);
        let onto = dy.atan2(dx);
        let first = onto + (FIRING_CLEAR_BY / dx.hypot(dy)).asin();
        game.image().the_player_is_at(FIRING_ONTO);
        for frame in 0..3 {
            let angle = first - ITS_BEAMS_TURN * frame as f32;
            game.image()
                .an_enemy_fires_a_beam(ITS_BEAMS_SLOT, beams_middle(angle), angle, 1);
            game.image().the_player_is_at(FIRING_ONTO);
            game.frames(1);
        }
        // Away from it being up and to the right, the way it is turning from, and towards it the other two.
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & (UP | RIGHT) != 0 && pressed & (LEFT | DOWN) == 0,
            "the rules did not move away from a laser turning onto them, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And out of its beam, round the field away from it corner by corner**: along the low line to the corner on
/// the side the run is on, then up that side — the angle the laser has to turn through to follow, which is
/// what the wiki's `左上、左下、右上、左下` leads it by. Along the floor alone, a run of the real game was put
/// into the bottom-left corner and hit there. **And between waves, up at the nearer upper corner**, which is
/// where the next laser is aimed at and so where the walk round starts: from the floor it had half of it,
/// and a run was pushed on past the corner to the field's own top-right and hit there.
#[test]
fn the_rules_of_th08_lead_the_following_laser_of_final_spark_round_the_field() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-lead-the-following-laser",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: out_of_the_following_laser, spell: {FINAL_SPARK} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().enemies_alive(ITS_BEAMS_SLOT + 1);
        game.image().a_boss_is_fighting(0, MARISA, 1000);
        game.image().an_enemy_has_no_body(ITS_BEAMS_SLOT - 1);
        game.image().an_enemy_has_no_body(ITS_BEAMS_SLOT);
        game.image().a_card_is_up(FINAL_SPARK);
        // Firing straight down from her, the run clear of it on her right at the low line: on to the corner
        // at the low line on that side.
        let down = std::f32::consts::FRAC_PI_2;
        game.image()
            .an_enemy_fires_a_beam(ITS_BEAMS_SLOT, beams_middle(down), down, 1);
        game.image().the_player_is_at((CLEAR_ON_HER_RIGHT, LOW));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 328,368"),
            "the rules did not go on to the corner at the low line:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_ne!(
            game.image().the_word_the_keys_read() & RIGHT,
            0,
            "the rules did not walk on to the corner:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And at that corner: up the side.
        game.image()
            .the_player_is_at((THE_CORNER_ON_HER_RIGHT, LOW));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 328,200"),
            "the rules did not go on up the side from the corner:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & UP != 0 && pressed & DOWN == 0,
            "the rules did not go up the side, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And between waves, the beam aimed and not firing and the run clear of it on her left: up to the
        // nearer upper corner, to be aimed at from there.
        game.image()
            .an_enemy_aims_a_beam(ITS_BEAMS_SLOT, beams_middle(down), down);
        game.image().the_player_is_at((CLEAR_ON_HER_LEFT, LOW));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 56,200"),
            "the rules did not wait at the upper corner on their side between waves:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And with the enemy that laid it gone, nothing aimed at all and the run on her right: the upper
        // corner on that side, and not back under her.
        game.image().enemies_alive(ITS_BEAMS_SLOT);
        game.image().a_boss_is_fighting(0, MARISA, 1000);
        game.image().an_enemy_has_no_body(ITS_BEAMS_SLOT - 1);
        game.image().the_player_is_at((CLEAR_ON_HER_RIGHT, LOW));
        let from = game.log().written();
        game.frames(2);
        assert!(
            game.log().said_since(
                from,
                "wanted the place the card's own dodge puts it at 328,200"
            ),
            "the rules went back to their own errand with no laser up between waves:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under `「パゼストバイフェニックス」` the rules a recipe has dodge it `round_the_field` go on round the field
/// clockwise, corner by corner, whatever the boss does**: she follows the run about and fires from where she
/// is, so waiting under her is waiting where her fire comes from — which is where one Extra was hit, standing
/// still under her with every move clear.
#[test]
fn the_rules_of_th08_go_round_the_field_through_possessed_by_phoenix() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-round-the-field",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: round_the_field, spell: {POSSESSED_BY_PHOENIX} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_card_is_up(POSSESSED_BY_PHOENIX);
        // At the bottom-left corner: up the left side, and not under her.
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 1000);
        game.image().the_player_is_at(ROUND_THE_FIELDS_CORNERS[3]);
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 56,200"),
            "the rules did not go on up the left side:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & UP != 0 && pressed & DOWN == 0,
            "the rules did not go up, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And at the top-left one: across the top.
        game.image().the_player_is_at(ROUND_THE_FIELDS_CORNERS[0]);
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 328,200"),
            "the rules did not go on across the top:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & RIGHT != 0 && pressed & LEFT == 0,
            "the rules did not go right, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under `「蓬莱人形」` the rules a recipe has dodge it `round_a_small_square` go round a square 48 across under the
/// boss clockwise, corner by corner, at the whole step**: its familiars stay on the run and fire from where they
/// are, so what gets through is never being still — the strategy wiki's `妹紅の前で小さく時計回りにちょん避け` —
/// and the focus key a calm frame under a boss would have down is the half step that was hit three times.
#[test]
fn the_rules_of_th08_go_round_a_small_square_through_hourai_doll() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-round-a-small-square",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: round_a_small_square, spell: {HOURAI_DOLL} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_card_is_up(HOURAI_DOLL);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        // The norm met, which is what has a calm frame under a boss hold the focus key otherwise.
        game.image().the_stage_has_its_time_orbs(2_500, 2_500);
        // **Part way along an edge and not on a corner**, a twelfth of the square's side short of the corner
        // behind it. Along its top: on to the top-right corner, unfocused.
        game.image().the_player_is_at((MIDDLE - 12.0, LOW - 48.0));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 216,320"),
            "the rules did not go on across the square's top:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & RIGHT != 0 && pressed & LEFT == 0 && pressed & FOCUS == 0,
            "the rules did not go right at the whole step, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And along its bottom: on to the bottom-left corner.
        game.image().the_player_is_at((MIDDLE + 12.0, LOW));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 168,368"),
            "the rules did not go on along the square's bottom:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & LEFT != 0 && pressed & RIGHT == 0 && pressed & FOCUS == 0,
            "the rules did not go left at the whole step, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And on from a corner of that square no step brings them any nearer to**, which is where the real game's
/// Extra stood for the last `63` frames of four attempts running and was hit: `1.2` short of the square's
/// top-left corner and `1.4` under it, every move open to it landing further from the corner than standing did.
/// Standing still there is standing where the familiars fire from.
///
/// A launch of its own, so that no step chosen for another place is still to be taken when the run is put there.
#[test]
fn the_rules_of_th08_go_on_from_a_corner_of_the_small_square_no_step_brings_them_nearer_to() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-on-from-a-corner-they-have-reached",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: round_a_small_square, spell: {HOURAI_DOLL} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_card_is_up(HOURAI_DOLL);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().the_stage_has_its_time_orbs(2_500, 2_500);
        game.image()
            .the_player_is_at((MIDDLE - 24.0 - 1.2, LOW - 48.0 + 1.4));
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 216,320"),
            "the rules stood by a corner they had reached:\n  {}",
            game.log().lines().join("\n  "),
        );
        let pressed = game.image().the_word_the_keys_read();
        assert!(
            pressed & RIGHT != 0 && pressed & LEFT == 0,
            "the rules did not go on across the square's top, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under `禁薬「蓬莱の薬」` the rules a recipe has dodge it `without_the_items` go for no item**, where the same point
/// item across the field is gone for with the card not up: Eirin lets them fall down the column her fire is
/// thickest in, and a run that went for them was one crushed at the floor under her.
#[test]
fn the_rules_of_th08_go_for_no_item_through_the_hourai_elixir() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-for-no-item",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: without_the_items, spell: {HOURAI_ELIXIR} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().an_item_is_at((MIDDLE + 100.0, LOW));
        game.image().the_item_is_of_class(A_POINT);
        game.frames(2);
        let log = game.log();
        assert!(
            log.said("autoplay: wanted a point item at 292,368")
                && game.image().the_word_the_keys_read() & RIGHT != 0,
            "the rules did not go for a point item with no card up:\n  {}",
            log.lines().join("\n  "),
        );
        // **A frame for the card to be read up on**, which is the frame the way is taken up after.
        game.image().a_card_is_up(HOURAI_ELIXIR);
        game.frames(1);
        let before = log.lines().len();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.frames(2);
        let since: Vec<String> = log.lines().split_off(before);
        assert!(
            log.said(&format!(
                "card {HOURAI_ELIXIR} is up, and the rules dodge it without_the_items"
            )) && !since
                .iter()
                .any(|line| line.contains("wanted a point item"))
                && game.image().the_word_the_keys_read() & RIGHT == 0,
            "the rules went for a point item through the card:\n  {}",
            since.join("\n  "),
        );
    });
}

/// **And a 1up gone for all the same**: a way that goes for no item is a way that keeps the run off the column the
/// points fall down, and a 1up is worth the walk wherever it falls — th10's Extra lost the one its midboss drops,
/// which falls into the stretch of midstage 2 its recipe dodges `without_the_items`.
#[test]
fn the_rules_of_th08_go_for_a_1up_through_the_hourai_elixir_dodged_without_the_items() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-for-a-1up-without-the-items",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: without_the_items, spell: {HOURAI_ELIXIR} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(HOURAI_ELIXIR);
        game.image().an_item_is_at((MIDDLE + 100.0, LOW));
        game.image().the_item_is_of_class(A_1UP);
        game.frames(3);
        let log = game.log();
        assert!(
            log.said(&format!(
                "card {HOURAI_ELIXIR} is up, and the rules dodge it without_the_items"
            )) && log.said("autoplay: wanted a 1up at 292,368")
                && game.image().the_word_the_keys_read() & RIGHT != 0,
            "the rules did not go for a 1up through the card:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **Under `難題「龍の頸の玉　-五色の弾丸-」` the rules a recipe has dodge it `side_to_side` go from one side of the
/// low line to the other and back**: its lasers are aimed at the run, so what gets through is never being where the
/// last one was aimed. On to the right-hand end, back from it once there, on past the middle the way they were
/// going, back from the left-hand end once the dodging has taken them past it to the wall — each end as far in
/// from its wall as a way's place is held — and back from wherever they have stood still for twenty of the card's
/// frames, the way on being shut.
#[test]
fn the_rules_of_th08_go_from_side_to_side_through_the_dragons_neck_jewel() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-go-from-side-to-side",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: side_to_side, spell: {DRAGONS_NECK_JEWEL} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_card_is_up(DRAGONS_NECK_JEWEL);
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        let log = game.log();
        for (at, end, way, pressed) in [
            ((MIDDLE, LOW), "328,368", "on to the right", RIGHT),
            ((330.0, LOW), "56,368", "back from the right", LEFT),
            ((MIDDLE, LOW), "56,368", "on past the middle", LEFT),
            ((20.0, LOW + 40.0), "328,368", "back from the left", RIGHT),
        ] {
            let before = log.lines().len();
            game.image().the_player_is_at(at);
            game.frames(2);
            let since: Vec<String> = log.lines().split_off(before);
            let wanted = format!("wanted the place the card's own dodge puts it at {end}");
            let word = game.image().the_word_the_keys_read() & (LEFT | RIGHT);
            assert!(
                since.iter().any(|line| line.contains(&wanted)) && word == pressed,
                "the rules did not go {way} from {at:?}, pressing {word:#05x}:\n  {}",
                since.join("\n  "),
            );
        }
        // **And back from where they have stood twenty of the card's frames**, the way on shut: going right,
        // and put back in the middle on every frame of the card's clock.
        let before = log.lines().len();
        for frames in 0..24 {
            game.image().the_boss_has_attacked_for(0, 1000 + frames);
            game.image().the_player_is_at((MIDDLE, LOW));
            game.frames(1);
        }
        let since: Vec<String> = log.lines().split_off(before);
        let wanted = |end: &str| {
            since.iter().position(|line| {
                line.contains(&format!(
                    "wanted the place the card's own dodge puts it at {end}"
                ))
            })
        };
        assert!(
            matches!((wanted("328,368"), wanted("56,368")), (Some(right), Some(left)) if right < left),
            "the rules did not turn back from where they stood:\n  {}",
            since.join("\n  "),
        );
    });
}

/// **Under `「インペリシャブルシューティング」` the rules a recipe has dodge it `imperishable_shooting` keep to their
/// own errand until its last wave and wait at the bottom-right corner through that**: its waves come at the same
/// frames of the card every launch, and the last from `4003` — see `orb_bot::cards`.
#[test]
fn the_rules_of_th08_dodge_imperishable_shooting_by_its_schedule() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-dodge-imperishable-shooting",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: imperishable_shooting, spell: {IMPERISHABLE_SHOOTING} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, LOW));
        game.image()
            .a_boss_is_fighting(0, (MIDDLE, A_RINGS_CENTRE), 6000);
        game.image().a_card_is_up(IMPERISHABLE_SHOOTING);
        // Before the last wave: under her, the rules' own errand.
        game.image()
            .the_boss_has_attacked_for(0, BEFORE_ITS_LAST_WAVE);
        game.frames(2);
        assert!(
            game.log().said("wanted a boss at 192,368")
                && !game
                    .log()
                    .said("wanted the place the card's own dodge puts it"),
            "the rules left their own errand before the card's last wave:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And in it: the bottom-right corner.
        game.image().the_boss_has_attacked_for(0, IN_ITS_LAST_WAVE);
        game.frames(2);
        assert!(
            game.log()
                .said("wanted the place the card's own dodge puts it at 352,432"),
            "the rules did not go to the bottom-right corner through the card's last wave:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Under `狂符「幻視調律」` the rules a recipe has dodge it `visionary_tuning` climb through its lattice while it is
/// hidden**, up to the middle forty below the band it is born in — none of it can hit while hidden, and shown it
/// comes down on the run for a hundred frames at a time: see `orb_bot::cards`. **And dodge it again for the
/// last twenty frames of a hide**, a hidden bullet being shown where it has got to.
#[test]
fn the_rules_of_th08_climb_through_visionary_tunings_lattice_while_it_is_hidden() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-rules-climb-through-visionary-tuning",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: visionary_tuning, spell: {VISIONARY_TUNING} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().the_player_is_at((MIDDLE, UNDER_A_ROW));
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(VISIONARY_TUNING);
        // A row of it across the whole field above the run, hidden, its boxes overlapping.
        for (slot, x) in (A_ROWS_SLOT..).zip((8..=376).step_by(4)) {
            game.image()
                .a_bullet_is_at(slot, (x as f32, A_ROW), (0.0, 0.0), (4.0, 4.0));
            game.image().the_bullet_is_hidden(slot);
        }
        // Late in a hide: kept below it.
        game.image().the_boss_has_attacked_for(0, LATE_IN_A_HIDE);
        game.frames(30);
        let (_, y) = game.image().where_the_player_is();
        assert!(
            y > A_ROW + 4.0,
            "the rules walked into the hidden row with it shown in twenty frames, at y = {y}:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Early in one: through it, and up to the place.
        game.image().the_boss_has_attacked_for(0, EARLY_IN_A_HIDE);
        game.frames(60);
        let (x, y) = game.image().where_the_player_is();
        assert!(
            (x - MIDDLE).abs() <= 4.0 && (y - VISIONARY_TUNINGS_PLACE).abs() <= 4.0,
            "the rules did not climb through the hidden row to 192,290, standing at {x},{y}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And no bomb spent under it on a stretch with nothing open**, which its lattice reads as on most of its
/// frames and is got through all the same — where the same stretch once the card is over is bombed out of, as
/// it is anywhere: see `the_bot_playing_th07`'s `a_place_nothing_has_been_open_in_is_bombed_out_of`.
#[test]
fn the_rules_of_th08_spend_no_bomb_on_a_stretch_of_visionary_tuning_with_nothing_open() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-no-bomb-on-a-stretch-of-visionary-tuning",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: visionary_tuning, spell: {VISIONARY_TUNING} }}\n\
                     - play_until: {{ frames: 1000 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        game.image().a_boss_is_fighting(0, (MIDDLE, 100.0), 1000);
        game.image().a_card_is_up(VISIONARY_TUNING);
        // Past the frames a stretch is held apart from the launch by, as th07's case is.
        game.frames(300);
        game.image().the_boss_has_attacked_for(0, WHILE_IT_IS_SHOWN);
        // **Two closing on the player from a distance its own next frame keeps more room than the slack at**, put
        // back every frame with the player — th07's case's place, the two wider than the field so that no move
        // is out of the band between them sideways, and each new in its slot every frame so that it is read by
        // the step it claims rather than as one a script holds where it is.
        let size = (A_STEP * 100.0, A_STEP * 24.0);
        let apart = size.1 / 2.0 + A_STEP + A_STEP;
        let shut_in = || {
            game.image().the_player_is_at((MIDDLE, LOW));
            game.image().a_bullet_is_new_in_its_slot(
                SLOTS[0],
                (MIDDLE, LOW - apart),
                (0.0, 1.0),
                size,
            );
            game.image().a_bullet_is_new_in_its_slot(
                SLOTS[1],
                (MIDDLE, LOW + apart),
                (0.0, -1.0),
                size,
            );
        };
        for _ in 0..70 {
            shut_in();
            game.frames(1);
        }
        game.one_frame_to_drain_the_log();
        assert!(
            game.log()
                .said("running with nothing open under a lattice that reads so and is got through")
                && !game.log().said("a bomb spent after"),
            "the rules spent a bomb on a stretch of the card with nothing open:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And the same place with the card over, once a stretch is due to be said again.
        game.image().no_card_is_up();
        for _ in 0..300 {
            shut_in();
            game.frames(1);
        }
        game.one_frame_to_drain_the_log();
        assert!(
            game.log().said("a bomb spent after"),
            "the rules spent no bomb on the same place with the card over:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The card and where it is waited at**, by its record's own number on Normal: `290`, `142` above the floor.
const VISIONARY_TUNING: usize = 105;
const VISIONARY_TUNINGS_PLACE: f32 = 290.0;
/// **And three frames of it by the boss's attack clock**, which the laid-out game moves on a frame at a time:
/// the first of a shown hundred, the first of a hide, and twenty before that hide ends at `640`.
const WHILE_IT_IS_SHOWN: i32 = 441;
const EARLY_IN_A_HIDE: i32 = 541;
const LATE_IN_A_HIDE: i32 = 621;
/// **The row across the field and where the run stands under it**, in slots of their own past [`SLOTS`].
const A_ROW: f32 = 400.0;
const UNDER_A_ROW: f32 = 420.0;
const A_ROWS_SLOT: usize = 400;
/// The rules' whole step.
const A_STEP: f32 = 4.0;

/// **The card whose familiars stay on the run**, by the number the Extra's recipe names it with.
const HOURAI_DOLL: usize = 202;

/// **And the card whose lasers are aimed at the run**, by the number Kaguya's clear names it with.
const DRAGONS_NECK_JEWEL: usize = 152;

/// **And Eirin's Last Spell**, by its record's own number, and the class of a point item — see `sight::worth`.
const HOURAI_ELIXIR: usize = 144;
const A_POINT: u8 = 1;

/// **The card and two frames of it**, one before its last wave and one inside it.
const IMPERISHABLE_SHOOTING: usize = 204;
const BEFORE_ITS_LAST_WAVE: i32 = 2600;
const IN_ITS_LAST_WAVE: i32 = 4100;

/// **The card and the corners it is walked round by**: `48` in from either side of a field `8` to `376`
/// across, half way up it at `200` and at the low line.
const POSSESSED_BY_PHOENIX: usize = 201;
const ROUND_THE_FIELDS_CORNERS: [(f32, f32); 4] =
    [(56.0, 200.0), (328.0, 200.0), (328.0, LOW), (56.0, LOW)];

/// Where a beam out of Marisa along `angle` has its enemy — the middle of its box, which is where this game
/// keeps it — see `th08::BEAM_ANCHOR_ALONG`.
fn beams_middle(angle: f32) -> (f32, f32) {
    (
        MARISA.0 + ITS_BEAMS_HALF_LENGTH * angle.cos(),
        MARISA.1 + ITS_BEAMS_HALF_LENGTH * angle.sin(),
    )
}

/// **The card and its beam**, as the real game has them: the card's number, where Marisa fires it from, the
/// slot of the enemy it is laid by, half its length, and how fast it turns once it fires.
const FINAL_SPARK: usize = 97;
const MARISA: (f32, f32) = (161.0, 103.0);
const ITS_BEAMS_SLOT: usize = 2;
const ITS_BEAMS_HALF_LENGTH: f32 = 295.0;
const ITS_BEAMS_TURN: f32 = 0.0036;
/// Where the run is put: `83.7` off the warning's middle as the real game had it, and, for the beam firing,
/// clear of its `120` by twenty with the beam turning onto it.
const WHERE_THE_RUN_WAS: f32 = 84.0;
const FIRING_ONTO: (f32, f32) = (330.0, LOW);
const FIRING_CLEAR_BY: f32 = 140.0;
/// And where the run is put for the walk round the field: clear of the beam straight down from her by `19`,
/// and the corner on that side, which is `48` in from the field's own right at `376`.
const CLEAR_ON_HER_RIGHT: f32 = 300.0;
const CLEAR_ON_HER_LEFT: f32 = 30.0;
const THE_CORNER_ON_HER_RIGHT: f32 = 328.0;

/// **The card and its laser**, as the real game has them: the card's number, where Mokou fires it from, and
/// the laser's own numbers off `[laser+0x554]` from frame to frame, `+0x570`, `+0x574`, `+0x55c` and `+0x564`.
const HONEST_MANS_DEATH: usize = 198;
const MOKOU: (f32, f32) = (192.0, 128.0);
const ITS_TURN: f32 = std::f32::consts::PI / 400.0;
const ITS_WARNING: i32 = 60;
const ITS_REACH: f32 = 512.0;
const ITS_WIDTH: f32 = 16.0;
/// For how many of the frames it sweeps on for once live the test follows it — a third of its second phase's
/// `120`, which is `18°` of sweep.
const LIVE_FOR: i32 = 40;
