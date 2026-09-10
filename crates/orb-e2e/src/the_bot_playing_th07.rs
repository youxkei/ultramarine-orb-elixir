//! **orb playing th07 itself, so that a run to be picked up out of a file costs nobody an afternoon.**
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
//! The rules are th07 on Normal, and dodging comes before all of them:
//!
//! - the shot held, let go for a frame every second so that a screen waiting for a press gets one, and Ctrl
//!   while a dialogue is up
//! - the bomb the frame the player is hit, which is th07's own deathbomb — [`PLAYER_STATE`] stands at 2 while
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

/// th07's own first two buttons, as `Th07` has them: shoot and bomb.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;
/// The one that skips a dialogue, which is Ctrl, and the one that slows the player down, which is Shift.
const SKIP: u16 = 0x100;
const FOCUS: u16 = 0x004;
/// How often the shot is let go for a frame, as `bot::SHOT_LET_GO_EVERY` has it.
const SHOT_LET_GO_EVERY: u32 = 60;

fn playing(name: &str) -> Box<Fake> {
    // A recipe of no steps, which is a run played from its first frame: what these two are about is the rules
    // and not the screens on the way in.
    playing_to(name, "[]")
}

/// And to a recipe of its own, which is what the rules a step *changes* are read against — see
/// [`a_chapter_drawn_to_be_hit_in_is_not_dodged_in`].
fn playing_to(name: &str, recipe: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
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

/// The shot is held on every frame of a stage, which is the whole of what a run of th07 does with it.
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
/// between stage 1's last chapter and stage 2's first.
///
/// Which is a rule about the *edge* and not about the fire rate. A frame in every
/// [`the_shot_is_let_go_every`](Self) is a shot in sixty, and th07's own shot fires off the player's timer
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
/// updates a bomb is still a deathbomb.
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
        // **More than the frame of the hit**, the line being one the frame loop writes in its own slack rather
        // than where it is worked out — see `detail!`.
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            BOMB,
            "the bot did not bomb the frame the player was hit:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **With nothing open, the move whose own next frame is clear of everything**, over one that is hit on fewer
/// of the frames looked at and already inside something on the next.
///
/// The next frame is the only one the player is certain to reach, and twelve more frames of choosing is what a
/// move that reaches it buys: a run of th06c's Extra died on the other order, taking a move `-3.2` into
/// something on its own next frame because it was hit on four of the thirteen, over one with `1.8` of room
/// there and hit on all thirteen.
#[test]
fn with_nothing_open_the_bot_takes_the_move_whose_next_frame_is_clear() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-keeps-its-next-frame");
        game.frames(30);
        game.image().the_player_is_at(MIDDLE, LOW);
        // **Two bigger on both axes than every frame looked at can cross**, above and below: every one of
        // the nine is hit, and the sideways ones are left a unit of room on their own next frame where the
        // rest are inside one of the two.
        let closing = 1.0;
        let size = (A_STEP * 32.0, A_STEP * 24.0);
        let apart = size.1 / 2.0 + closing * 2.0 + A_STEP;
        game.image()
            .a_bullet_of_size_is_at(500, MIDDLE, LOW - apart, (0.0, closing), size);
        game.image()
            .a_bullet_of_size_is_at(501, MIDDLE, LOW + apart, (0.0, -closing), size);
        // **And what makes the two sides differ.** To the left, standing still, something the walk runs deep
        // into by the last of the frames looked at and is nowhere near on the first — so going left is the
        // move with the least room of the thirteen. To the right, something crossing the landing on the very
        // next frame and gone by the one after — so going right is the move with the most room of the
        // thirteen and a hit on the next.
        game.image().a_bullet_of_size_is_at(
            502,
            MIDDLE - A_STEP * 18.0,
            LOW,
            (0.0, 0.0),
            (A_STEP * 20.0, size.1),
        );
        // **Far enough left that where it stands now is clear of the left landing**, which the judgement
        // reads as well as where its step takes it: a landing has to be clear of both places, so a crossing
        // bullet parked on the move this test is about would shut that move rather than the one it is for.
        game.image().a_bullet_of_size_is_at(
            503,
            MIDDLE - A_STEP * 4.0,
            LOW,
            (A_STEP * 5.0, 0.0),
            (A_STEP * 2.0, A_STEP * 2.0),
        );

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & (LEFT | RIGHT),
            LEFT,
            "the bot took a move already inside something on its own next frame:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a bomb where nothing has been open for a second running**, which is a place lost rather than a
/// frame lost: every one of those frames was the best of its own nine, so nothing in the ranking gets the
/// player out of where it is. th06c's Extra boss took a run's hit after `169` such frames, the player in
/// the thickest box of the five the whole time and the rules wanting the emptiest on every one of them.
///
/// **And not on one of them**, which is the judgement's own pessimism: `steer::SHUT_IN_FOR` has what a single
/// frame of it cost.
///
/// **This and the deathbomb are the only two bombs there are**, and what tells them from the frame before a
/// hit — which spends nothing — is what the bomb buys: that one is the same hit paid a frame earlier, where
/// a stretch is a place, and clearing the screen is the one thing that puts a run somewhere it can play
/// from. `steer`'s own record of these frames has the climb that measured it.
#[test]
fn a_place_nothing_has_been_open_in_is_bombed_out_of() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-place-nothing-was-open-in");
        game.frames(300);
        // **Two closing on the player from a distance its own next frame keeps more room than the slack at**,
        // where the last of the frames looked at is inside it — so nothing is open to any of the nine and the
        // move chosen is not hit on its own next frame, which is the state a single frame of is nothing.
        //
        // **Put back every frame**, both of them and the player: a stretch of it is the whole subject, and
        // either of them left to close turns it into the corner the other bomb answers within two frames.
        let closing = 1.0;
        let size = (A_STEP * 32.0, A_STEP * 24.0);
        let apart = size.1 / 2.0 + A_STEP + A_STEP;
        let shut_in = || {
            game.image().the_player_is_at(MIDDLE, LOW);
            game.image()
                .a_bullet_of_size_is_at(500, MIDDLE, LOW - apart, (0.0, closing), size);
            game.image()
                .a_bullet_of_size_is_at(501, MIDDLE, LOW + apart, (0.0, -closing), size);
        };

        // A sixth of a second of it, which is not a place lost.
        for _ in 0..10 {
            shut_in();
            game.frames(1);
        }
        game.one_frame_to_drain_the_log();
        assert!(
            !game.log().said("frame(s) running with nothing open"),
            "the bot called ten frames with nothing open a place lost:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And a second of it, which is. **Read off the line and not off the word**, the word the game acts
        // on being the one the frame before it left: the bomb is pressed on the frame the stretch reaches a
        // second, and by the frame that word is readable the stretch has been reset and the next word has
        // no bomb in it. The line says which of the three the frame was, which is what this is about.
        for _ in 0..60 {
            shut_in();
            game.frames(1);
        }
        game.one_frame_to_drain_the_log();
        assert!(
            game.log().said("a bomb spent after"),
            "the bot did not spend a bomb on a place nothing had been open in for a second:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And where not one of the nine reaches its own next frame, the shallowest of the overlaps.**
///
/// The frames after the next one are frames that never arrive, so a move hit on two of the thirteen and six
/// units inside something on the next is worth less than one hit on all thirteen and a unit inside: `room`, the
/// least of the thirteen, cannot tell them apart and `hits()` has them the wrong way round. A run of th06c's
/// Extra stood still into a beam `-6.0` deep over a step aside `-2.0` deep for exactly that reason.
#[test]
fn with_nothing_reaching_its_next_frame_the_bot_takes_the_shallowest_overlap() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-the-shallowest");
        game.frames(30);
        game.image().the_player_is_at(MIDDLE, LOW);
        // **Two that put every one of the nine a unit inside something on its own next frame**, above and
        // below and wider on both axes than the frames looked at can cross — so nothing is open, nothing
        // reaches its next frame, and every move is hit on all thirteen.
        let closing = 1.0;
        let size = (A_STEP * 32.0, A_STEP * 24.0);
        let apart = size.1 / 2.0 + A_STEP;
        game.image()
            .a_bullet_of_size_is_at(500, MIDDLE, LOW - apart, (0.0, closing), size);
        game.image()
            .a_bullet_of_size_is_at(501, MIDDLE, LOW + apart, (0.0, -closing), size);
        // **And one to the left, drifting right**, which is what makes the three middle moves differ on their
        // own next frame and on nothing else: going left is seven units into it, standing still three, and
        // going right is clear of it — so right is the shallowest overlap of the nine while sharing the least
        // room of the thirteen with left.
        game.image().a_bullet_of_size_is_at(
            502,
            MIDDLE - A_STEP * 3.0,
            LOW,
            (closing, 0.0),
            (A_STEP * 5.0, size.1),
        );

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & (LEFT | RIGHT),
            RIGHT,
            "the bot took the deeper of two overlaps on its own next frame:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a corner where nothing is open and the move it chose has less room left on its own next frame than
/// a bomb is owed at is written down, with the bomb kept for the window.**
///
/// **Under that and not overlapping.** A tenth of a unit of room on the next frame is not room, and a corner
/// read only at the overlap is one read too late: th06c's Extra boss took a run's first hit on such a frame
/// with three bombs still in hand. What the number is and why it is neither nought nor the slack a clear move
/// is judged with is `steer::ROOM_A_BOMB_IS_OWED_AT`.
///
/// **And what the bomb is for is the hit the game has counted**, which such a frame is one frame short of:
/// spending it here buys nothing the deathbomb does not and takes the stock the window needs, so the word
/// read back carries the line and not the bomb.
#[test]
fn a_corner_with_only_the_slack_left_is_written_down_unbombed() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-corner-with-the-slack-left");
        // **Past the frames one corner is held from the next by**, which a launch counts from nought: inside
        // them no corner is written down at all, so one put up before them reads as one the rules passed over.
        game.frames(300);
        game.image().the_player_is_at(MIDDLE, LOW);
        // **One above and one below, each bigger on both axes than every frame looked at can cross**, so
        // that not one of the nine is out of the way of both: a box a diagonal passes through and comes out
        // the far side of is a move with frames of its own to spare, and that move is then the one chosen
        // with its own next frame already over the boxes — which is the reading this test is here instead
        // of.
        //
        // **And closing a unit a frame onto a quarter of one**, which is a next frame under
        // `steer::ROOM_A_BOMB_IS_OWED_AT` and still clear of the boxes: at nought the next frame is already
        // a hit, which is the reading this one is here instead of, and at the whole of
        // `steer::ROOM_TO_SPARE` it is a frame nothing was going to touch, which is the other.
        let closing = 1.0;
        let a_quarter = 0.25;
        let size = (A_STEP * 32.0, A_STEP * 24.0);
        let apart = size.1 / 2.0 + A_STEP + closing + a_quarter;
        game.image()
            .a_bullet_of_size_is_at(500, MIDDLE, LOW - apart, (0.0, closing), size);
        game.image()
            .a_bullet_of_size_is_at(501, MIDDLE, LOW + apart, (0.0, -closing), size);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            0,
            "the bot spent a bomb on a corner rather than keeping it for the window:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.one_frame_to_drain_the_log();
        assert!(
            game.log().said("of room on the move's own next frame"),
            "the bot did not write down a corner it had only the slack left in:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// th07's own d-pad bits, as `Th07` has them.
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
/// All four, for an assertion about a frame nothing should have been steered on.
const PAD: u16 = UP | DOWN | LEFT | RIGHT;

/// The kinds of item these place, as the table at 0x433a5a indexes them: 0 steps the power up by one and 2 by
/// eight — the big power item — 5 is the life and 3 the bomb, 1 is worth a score off the chain and 7 fills the
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
const LOW: f32 = 384.0;
const MIDDLE: f32 = 192.0;
/// And how far off either side edge the x they wait on is held, as `bot::IN_FROM_THE_SIDE` has it.
const IN_FROM_THE_SIDE: f32 = 48.0;
/// And the top of the band it moves in, which is the bottom third of that field.
const BAND_TOP: f32 = FIELD.1 + FIELD.3 * 2.0 / 3.0;
/// What one frame of holding a direction moves the player by in these, straight.
const A_STEP: f32 = 4.0;
/// **The widest bullet th07 makes**, which is the large bullet: the spawn writes each kind's own size into
/// `+0xb7c` and `+0xb80`, and of the sizes it writes — 4, 5, 6, 8, 10 — this is the one at 0x4275ca.
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
    // **And bombs in hand, which is what a run has**: the block those numbers live in is laid out zeroed,
    // and the rules now ask whether the bomb they are reaching for is one there is — see `Frame::bombs`.
    // Without this every case about bombing would be a case whose run had none, and the two that expect one
    // would be measuring the count rather than the corner.
    game.image().has_lives_and_bombs(2.0, 3.0);
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

/// And **the big one over a small one wherever both are on the screen**, whatever the distances: th07's
/// big power item is eight steps of power against the small one's one, so a stage that drops one drops the
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

/// **An enemy is dodged as much as a bullet**, its body being as solid as one in th07 — and it is what a run
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
            // **Up and not down**, which is a tie and is decided by the order of `steer::WAYS`: the enemy is
            // level with the player, so stepping off its line costs the same either way and the item is the
            // same distance from both landings. That order has why up is the half it goes to.
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
/// bullet's path every time, and every hit it took was a bomb — bombs 3, 2, 1, 0 through stage 1's boss with the
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
/// exists to make: the first launch that played its way to stage 2 left `resume: not written — 0 frame(s) of buttons`
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

        // The first frame has nothing to compare with, so the beam is only where it points — wide of the player.
        game.image().a_laser_is_at(
            11,
            (MIDDLE, FIELD.1),
            straight_down - 0.05,
            (0.0, FIELD.3),
            A_STEP * 2.0,
        );
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left a beam that was pointing wide of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And now it has turned a little towards it. Where it points is still wide; where it is going is not.
        game.image()
            .the_laser_was_turned_to(11, straight_down - 0.04);
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
/// A launch that could not see them was hit eighteen times on stage 4's boss, and every one of those hits said the
/// move it was taken on had twenty units of room — which is not a hitbox got wrong but a hazard that is not in
/// the model at all.
///
/// One straight down the player's own column and wide enough to cover it, so the way out is sideways — and it
/// stays a thing to leave once it is fading, which is what a hit measured.
///
/// **A fading beam still hits.** This asserted the opposite, off a reading of the wrong walk: the phase at
/// `+0x4e8` is what the two *bullet* walks stop at — 0x4188eb and 0x424864, which decide whether a beam deletes
/// a bullet — and the player's own test is called from all three of the phase branches at 0x42657a. 0x426727 is
/// the phase 0 call, gated on the warning line's timer; 0x4267ec is phase 1's and 0x4269a5 is phase 2's, and neither
/// has a gate before it at all. A run was hit at `201.1,356.0` with every one of the nine moves reporting
/// between 70 and 99 units of room and the model holding `0 laser(s) … with 20 passed over for their phase`.
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
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed == LEFT || pressed == RIGHT,
            "the bot stood in a fading laser, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A warning line is left whatever frame its beam fires on**, which is why anybody moves off one: the beam is
/// coming, and where it will be is where the player should not be.
///
/// 0x4266dc gates the player's own laser test on the beam's count reaching the frame it fires on, so a warning
/// cannot *hit* — and counting the beam only from that frame is what this used to do. The frames looked at being
/// twelve, a beam firing in twenty of them was nothing at all to the model: a run stood inside a lattice of
/// warning lines until twelve frames before they went live, and came back with hits reading `2.3 of room` and
/// `3.3` with **twenty lasers left out of the model**.
///
/// What the timer buys instead is that the beam is judged by the room it leaves rather than shutting a move
/// outright — a move hit late beats one hit now — so the one lane a pattern leaves open is still a lane.
#[test]
fn a_warning_line_is_left_whatever_frame_its_beam_fires_on() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-reads-a-warning-line");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().a_laser_is_at(
            7,
            (MIDDLE, FIELD.1),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            A_STEP * 2.0,
        );

        // Firing long after every frame this looks at, and still a lane to be out of.
        game.image().the_laser_fires_in(7, 600);
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed == LEFT || pressed == RIGHT,
            "the bot stood in a beam's own lane for the beam not firing yet, pressing \
             {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same the moment it is about to fire.
        game.image().the_laser_fires_in(7, 2);
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed == LEFT || pressed == RIGHT,
            "the bot stood in a beam about to fire, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A warning line is crossed where the warning lines leave nowhere to go.**
///
/// Which is the other half of [`a_warning_line_is_left_whatever_frame_its_beam_fires_on`]: a beam still drawing
/// its warning cannot hit anything — 0x4266dc gates the game's own test on the timer reaching
/// [`LASER_LIVE_AT`] — so a move into one is worse than a move out of one and better than a move into a live
/// beam. Stage 6's last card fans twenty-two warnings out of a single point, and a bot that shut a move for each of
/// them had all nine shut: it stood where it was until the beams fired, and the hit that came out of it read
/// `nowhere open to it`.
#[test]
fn a_warning_line_is_crossed_where_they_leave_nowhere_to_go() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-warning-line-is-crossed");
        // Off to one side, so that what the bot wants is a place it has to move to reach.
        game.image().the_player_is_at(MIDDLE - 100.0, LOW);
        // A warning wide enough that every one of the nine lands inside it, so no move is out of one.
        game.image().a_laser_is_at(
            7,
            (MIDDLE, FIELD.1),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            FIELD.2 * 2.0,
        );
        game.image().the_laser_fires_in(7, 600);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot did not cross a warning that left it nowhere to go:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A beam swept round by a script is a third of the field to be out of, and not only a thing to dodge.**
///
/// Stage 6's turning beam is a line of bullets a script sweeps: nothing on one of them is a step, so where it will
/// be is not a thing the nine moves can be judged against — [`a_bullet_that_ignores_its_own_step_is_feared_rather_than_projected`]
/// is as far as one frame's dodging goes. What is left is where to stand, and the answer is the same as for a
/// third the fire is piled into: somewhere else. Three hits in one practice came out of standing in the arms and
/// dodging inside them.
#[test]
fn a_beam_a_script_sweeps_is_a_box_of_the_field_to_be_out_of() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-leaves-a-swept-third");
        // In the middle third, which with no boss and nothing to collect is where it would otherwise wait.
        game.image().the_player_is_at(MIDDLE, LOW);
        // An arm of them across the middle third, high enough up that none is a dodge of its own.
        let arm: Vec<f32> = (0..16).map(|n| FIELD.1 + 40.0 + n as f32 * 4.0).collect();
        for (slot, y) in arm.iter().enumerate() {
            game.image().a_bullet_is_at(slot, MIDDLE, *y, (0.0, 0.0));
        }
        game.frames(1);
        // And every one of them somewhere its own step could not have taken it, which is a script sweeping them.
        for (slot, y) in arm.iter().enumerate() {
            game.image()
                .a_bullet_is_at(slot, MIDDLE + A_STEP * 3.0, *y, (0.0, 0.0));
        }
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed & (LEFT | RIGHT) != 0,
            "the bot waited in the third a swept beam was crossing, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A hit names the nearest of every kind with no reach to fall outside of, and every laser slot that holds
/// anything.**
///
/// Which is what is left when a hit has been read every other way and the answer keeps coming back empty. Six
/// hits in three practices said `nothing was near it at all` with three hundred bullets on the screen and a beam
/// visibly across the player — so either something is in one of the arrays further off than the dump asks about,
/// or it is in a slot a gate of orb's own skips. Both of those are questions about numbers nobody has printed,
/// and neither is a question the arrays can be asked with a rule already applied to them.
#[test]
fn a_hit_names_the_nearest_of_each_kind_and_every_laser_slot_holding_anything() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-names-the-nearest-of-each");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Further off than a hit's own reach, so a dump that finds it is one with no reach.
        game.image()
            .a_bullet_is_at(41, MIDDLE + 120.0, LOW - 120.0, (0.0, 0.0));
        // And a slot the aliveness gate skips, holding geometry all the same.
        game.image().a_laser_is_at(
            9,
            (MIDDLE, FIELD.1),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            A_STEP,
        );
        game.image().the_laser_slot_is_not_alive(9);
        game.frames(1);
        game.image().the_player_has_been_hit();
        game.frames(4);
        let log = game.log();
        assert!(
            log.said("the nearest bullet is 41"),
            "the hit did not name the nearest bullet with no reach to fall outside of:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            log.said("laser slot 9 holds"),
            "the hit did not dump a laser slot the aliveness gate skips:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A beam that follows an enemy is every place that enemy has been, and each of them is a hit.**
///
/// Stage 6's white spiral is one enemy and the trail behind it: 0x4210c2 hands 0x420490 a box at
/// `enemy + 0x2f78 + n * 0x1c` with the enemy's own size, from the first of them every sixth, as far as the count
/// at `+0x4f34`. So an enemy read as one box is a beam read as its head alone — which is what four practices of
/// hits reading `nothing was near it at all` were, with the spiral plainly across the player.
#[test]
fn a_beam_that_follows_an_enemy_is_every_place_it_has_been() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-beam-follows-an-enemy");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The enemy itself right out of the way, so nothing but its trail can be what the bot moves for.
        game.image()
            .an_enemy_of_size_is_at(4, FIELD.0 + 8.0, FIELD.1 + 8.0, (8.0, 8.0));
        // And its trail led down to where the player stands. The seventh is the second the game tests, so the
        // places up to it are what a trail has to hold to be one.
        let trail: Vec<(f32, f32)> = (0..8)
            .map(|n| (MIDDLE, LOW - 40.0 + n as f32 * 5.0))
            .collect();
        game.image().the_enemy_has_been_at(4, &trail);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot held still under a beam of the places an enemy had been:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the bot keeps a distance from every one of those places, whatever the box each one is says.**
///
/// The box a node is tested as is the enemy's own size over one and a half and halved again — a couple of units
/// for stage 6's spiral — so a beam a few units off the player is one every move reads as clear right up to the
/// frame it is not. What was tried for that was weighting those boxes heavily in the fire each box of the field
/// holds, and it answers a different question: the counting says which of five boxes to *wait* in and nothing
/// about how near the beam the bot may stand inside one. Six hits across four practices were inside a spiral the
/// counting had already sent it away from.
///
/// So the trail is out of the counting and the clearance is a bound on the dodge: `bot::CLEAR_OF_A_BEAM` on
/// every side of each place the enemy has been.
#[test]
fn the_bot_keeps_its_distance_from_every_place_a_beam_has_been() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-beams-own-room");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The enemy itself right out of the way, so nothing but its trail can be what the bot moves for.
        game.image()
            .an_enemy_of_size_is_at(4, FIELD.0 + 8.0, FIELD.1 + 8.0, (8.0, 8.0));
        // A beam standing twenty-eight to the player's left, which is past everything the box of one node
        // reaches — the enemy's own size over one and a half and halved, the player's own box, and
        // `bot::ROOM_TO_SPARE` on top of both — and inside the room the bot is to keep. **Beside it rather than
        // above it**, so that the way clear is one direction and the answer cannot be a step the rules would have
        // taken anyway: the y the rules want is the y the player is already at, and with nothing near they press
        // nothing at all.
        //
        // **And a run of nodes the game tests exactly one of**, which is the second: it walks them from the first
        // every sixth, so seven places leave one box of fire in the box the player is standing in — under what
        // the counting leaves a box for, and a beam of two would be over it. So what moves the bot here is the
        // distance and nothing else.
        let trail: Vec<(f32, f32)> = (0..7)
            .map(|n| (MIDDLE - 28.0, LOW + 4.0 - n as f32 * 4.0))
            .collect();
        game.image().the_enemy_has_been_at(4, &trail);
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert_eq!(
            pressed & (LEFT | RIGHT),
            RIGHT,
            "the bot stood twenty-eight units off a beam it is to keep clear of, pressing \
             {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A big bullet is more of a third's fire than a small one.**
///
/// Counting them one apiece asks how many there are where what matters is how much of the band is taken: a large
/// bullet is six times a small one across, so a handful of them is a third with no way through and a handful of
/// small ones is a third to weave in. Three of the big ones here would not trip a count of heads and do trip a
/// count of widths.
#[test]
fn a_box_is_counted_by_the_width_of_its_fire_and_not_the_heads() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-third-is-counted-by-width");
        game.image().the_player_is_at(MIDDLE, LOW);
        // High in the band, so that nothing about them is a dodge of this frame.
        for slot in 0..3 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE + slot as f32 * 8.0,
                LOW - 60.0,
                (0.0, 0.0),
                (24.0, 24.0),
            );
        }
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed & (LEFT | RIGHT) != 0,
            "the bot waited in a third three large bullets had filled, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a beam is more of it than a bullet of its own size.**
///
/// A bullet crosses the band and is gone; the places an enemy has been are where they are and stay, so the third
/// a beam lies in has no run of good frames to wait for. Three of its boxes here are three bullets of the same
/// size by width alone, which is not enough to leave a third for — and are enough as a beam's.
#[test]
fn a_beam_is_more_of_a_boxs_fire_than_bullets_of_its_size() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-beam-is-more-of-the-fire");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .an_enemy_of_size_is_at(4, FIELD.0 + 8.0, FIELD.1 + 8.0, (8.0, 8.0));
        // Nineteen places, of which the game tests the first, the seventh and the thirteenth.
        let trail: Vec<(f32, f32)> = (0..19)
            .map(|n| (MIDDLE + (n % 3) as f32 * 8.0, LOW - 60.0))
            .collect();
        game.image().the_enemy_has_been_at(4, &trail);
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed & (LEFT | RIGHT) != 0,
            "the bot waited in a third three of a beam's boxes lay in, pressing \
             {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A hit says what every move came to, so that the one taken can be read against the ones left.**
///
/// Which is what the ranking cannot be argued about without. A practice was hit at `99.6,432.0` with nowhere open
/// to it, and the frames before it say the player walked from `410` to the floor at `432` over fifteen of them and
/// sat pinned there for fourteen more — so every one of those steps was chosen over a step across, and *why* is a
/// comparison between verdicts that nothing prints. Whether down won on when it was first hit or merely tied and
/// came earlier in the order is the difference between two different repairs.
#[test]
fn a_hit_says_what_every_move_came_to() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-says-all-nine");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(0, MIDDLE, LOW - 8.0, (0.0, 3.0));
        game.frames(1);
        game.image().the_player_has_been_hit();
        game.frames(4);
        let log = game.log();
        assert!(
            log.said("the move 0x000 was hit first on"),
            "the hit did not say what standing still came to:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            log.said(&format!("the move {DOWN:#05x} was hit first on")),
            "the hit did not say what going down came to:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **The field is cut into five across, so the box the bot moves to is a fifth of it and not a third.**
///
/// Which is a difference the pressed direction can be asked about: with the fire piled into the middle, the box
/// to wait in is the leftmost, and its centre is at a tenth of the width where five are cut and a sixth where
/// three are. A player standing between those two is sent one way by five boxes and the other by three.
///
/// Five rather than four so that there is a middle at all — the place an aimed shot converges on wants a box of its
/// own to be discounted in, and an even cut has none.
#[test]
fn the_field_is_cut_into_five_boxes_across() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-five-boxes-across");
        // Between the leftmost fifth's own middle, once held off the edge, and the leftmost third's.
        game.image().the_player_is_at(FIELD.0 + 56.0, LOW);
        // Piled into the middle, high in the band and far across the field, so that none of it is a dodge of
        // this frame and all of it is the counting.
        for slot in 0..15 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE + (slot % 5) as f32 * 4.0,
                LOW - 40.0 - (slot / 5) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot did not head for the leftmost of five boxes:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The box waited in is left for the emptiest whenever it is clearly fuller, and not only when it is the
/// fullest of them all.**
///
/// A run was hit at `248.3,432.0` boxed in at the bottom right with the whole of the bottom left plainly empty,
/// and the counting had seen it: the box the player was in held clearly more than the emptiest, and it was not
/// the *fullest* box on the screen — that was the pile up and to the left — so a rule asking whether it was left
/// it where it stood. What decides whether to move is what there is to move to.
#[test]
fn a_box_clearly_fuller_than_the_emptiest_is_left_even_where_another_is_fuller_still() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-fuller-box-is-left");
        // Standing in the middle, which is the box the rules would otherwise wait in.
        game.image().the_player_is_at(MIDDLE, LOW);
        // The fullest box is the rightmost, the middle holds clearly more than the leftmost, and the leftmost is
        // empty. High in the band so that none of it is a dodge of this frame.
        for slot in 0..40 {
            game.image().a_bullet_of_size_is_at(
                slot,
                FIELD.0 + FIELD.2 * 0.9,
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        for slot in 40..60 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE,
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot waited in a box clearly fuller than the emptiest:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The fire a box is counted by is everything in the bottom half of the field, not only the band the bot moves
/// in.**
///
/// A run was hit boxed in at the bottom right with the bottom left plainly empty, and what was descending on it
/// was a band of fire that had not reached the bottom third yet: counted only there, the box read as clear
/// enough to wait in until the band arrived, by which time there was nowhere to go. What decides where to wait
/// is what is coming, and the half of the field above the band is where it comes from.
#[test]
fn a_box_is_counted_by_the_fire_in_the_bottom_half_of_the_field() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-box-counts-the-bottom-half");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Above the bottom third and inside the bottom half, so only a count that reaches there sees it.
        let above = FIELD.1 + FIELD.3 * 0.6;
        for slot in 0..20 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE,
                above + (slot % 5) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed & (LEFT | RIGHT) != 0,
            "the bot waited under a band of fire the count could not see, pressing \
             {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The box the player is standing in is left, and not only the box it was heading for.**
///
/// Which is where following a boss takes it: the rule asked whether the *wanted* x was in the thickest box, and
/// a boss drifting across one is a boss the bot follows into it — the question never being put about the ground
/// under its own feet. A practice was hit at `138.0,409.5` with the boxes holding `15.0, 218.7, 58.0, 13.5,
/// 17.5`: the player had walked from `164` to `132` over forty frames, out of the box the boss was in and into
/// the fullest one on the screen, while the emptiest lay the other way.
#[test]
fn the_box_the_player_stands_in_is_left_and_not_only_the_one_it_was_heading_for() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-box-underfoot-is-left");
        // Standing in the box the fire is piled into, with the boss further the same way — so following the boss
        // and leaving the box are opposite moves, and only the ground underfoot sends it right.
        let thick = FIELD.0 + FIELD.2 * 0.3;
        game.image().the_player_is_at(thick, LOW);
        game.image().a_boss_is_fighting(1000, 600);
        game.image()
            .the_boss_is_at(FIELD.0 + FIELD.2 * 0.1, FIELD.1 + 40.0);
        // Badly worse than anywhere else, which is what it takes to walk out from under a boss.
        for slot in 0..80 {
            game.image().a_bullet_of_size_is_at(
                slot,
                thick,
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        // And a little in the two boxes to its left and middle, so that the emptiest lies the other way from the
        // boss rather than beside it.
        for slot in 80..86 {
            game.image().a_bullet_of_size_is_at(
                slot,
                FIELD.0 + FIELD.2 * if slot % 2 == 0 { 0.1 } else { 0.5 },
                LOW - 40.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot stood in the box the fire was piled into:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And of two boxes as empty as each other, the one the player has less field to cross to reach.**
///
/// The count is a measure of a screen in motion and the emptiest of five is settled by a tenth of a bullet, so
/// the bare least of them throws the place to wait a fifth of the field away on a difference nothing can act on:
/// six consecutive samples of one th06c Extra's walk wanted `56`, `56`, `266`, `56`, `118` and `56`, the two that
/// flipped reading `11, 13, 16, 11, 11` and `16, 13, 26, 13, 16`. A run given that spends the pattern crossing,
/// and the same walk goes on up out of the band it was waiting in to be hit at `176,202`.
///
/// Here the box the player stands in is one to leave, the two either side of it are piled too, and what is left
/// is a box one across holding a bullet or two against an empty one two across the other way — which the bare
/// least sends it past both piles to reach.
#[test]
fn the_nearer_of_two_boxes_as_empty_as_each_other_is_the_one_waited_in() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-nearer-of-two-empty-boxes");
        // In the middle, which is the box to be out of.
        game.image().the_player_is_at(MIDDLE, LOW);
        // Badly worse than anywhere else, so that the ground underfoot is what moves it.
        for slot in 0..30 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE,
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        // A pile in the leftmost and another in the second from the right, so that neither is a candidate and
        // both are fire to cross.
        for slot in 30..54 {
            game.image().a_bullet_of_size_is_at(
                slot,
                FIELD.0 + FIELD.2 * if slot < 42 { 0.1 } else { 0.7 },
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        // And a handful in the box one to the left, which leaves the rightmost emptier than it by a handful and
        // two boxes further off.
        for slot in 54..58 {
            game.image().a_bullet_of_size_is_at(
                slot,
                FIELD.0 + FIELD.2 * 0.3,
                LOW - 40.0 - (slot % 8) as f32 * 4.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot crossed two piles for a box a handful emptier:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the frame a climb out of the band is made on says what every move came to, the same as a hit
/// does.**
///
/// Which is the one frame of a climb anything can be read off. The two dumps beside this are a hit and a
/// second with nothing open, and a climb is neither: nine attempts of one Extra put three hits apiece above
/// the band's own top, always at the same three places, and read back from each of them the run spends sixty
/// frames going up with three or four of the nine open. So what says why upward was the move it had is the
/// nine judgements of the frame it left on, and until this nothing carried them — the frames before a hit
/// say what the rules wanted and how many were clear, and not which ones.
#[test]
fn the_frame_a_climb_out_of_the_band_is_made_on_says_what_every_move_came_to() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-climb-says-what-every-move-came-to");
        // Inside the band, which is the bottom third of the field — and for longer than one dump is held
        // apart from the next, so that the frame below is not the one that wait swallows.
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(300);
        game.one_frame_to_drain_the_log();
        assert!(
            !game.log().said("left the band at"),
            "a frame inside the band was written down as a climb out of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And above its top, which is where every one of those hits was.
        game.image()
            .the_player_is_at(MIDDLE, FIELD.1 + FIELD.3 / 4.0);
        game.frames(1);
        game.one_frame_to_drain_the_log();
        let log = game.log();
        assert!(
            log.said("left the band at") && log.said("the move 0x000 was"),
            "the climb out of the band said nothing of the nine moves:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And a dump says which boxes of the field a beam will be lying across the band in, which is the hazard
/// the fire in a box cannot count.**
///
/// A beam is its own array and not one of the hazards counted, so a screen with an arm across one side of the
/// field and nothing down the other reads as empty on both — and the frames a run climbs out of the band on
/// are frames an arm is lying across the strip it waits in. One of them, dumped at `76.3,292.2`, has the
/// three upward moves clear and the other six hit, with the nearest thing an arm `25` below the player
/// running down to the left across the band at `1.2` units a frame.
///
/// **Across and not along**, which is the whole of the measure and what the two attempts at counting a beam
/// as fire got wrong: the width it takes out of a box inside the band, against `steer::A_BOX_IS_BLOCKED_AT`.
/// An arm `17°` off the horizontal spans `146` units of x in there where a beam running down through the band
/// spans `7` however long it is — and a run moved for the second is
/// [`the_bot_leaves_a_laser_that_is_turning_onto_it`], which asks for no move at all.
#[test]
fn a_dump_says_which_boxes_a_beam_will_lie_across_the_band_in() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-which-boxes-a-beam-lies-across");
        // Inside the band for longer than one dump is held apart from the next.
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(300);

        // A beam running *down* through the band, which takes a few units of one box's width however long it
        // is. Thirteen frames of climbing reach `52` up from the waiting line, so it is put out of the nine's
        // own reach and what answers is the count and not a dodge.
        let across_the_band = LOW - A_STEP * 18.0;
        game.image().a_laser_is_at(
            11,
            (MIDDLE, FIELD.1),
            std::f32::consts::FRAC_PI_2,
            (0.0, FIELD.3),
            A_STEP * 3.0,
        );
        let along = game.log().written();
        game.image()
            .the_player_is_at(MIDDLE, FIELD.1 + FIELD.3 / 4.0);
        game.frames(1);
        game.one_frame_to_drain_the_log();
        assert!(
            game.log().said_since(
                along,
                "in each box from frame never, never, never, never, never"
            ),
            "a beam running down through the band was counted as lying across one:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And back inside it for the wait, then an arm lying across three of the five: from a hundred left of
        // the middle to a hundred right of it, which is the whole of the middle box and most of the two
        // beside it.
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(300);
        game.image().a_laser_is_at(
            11,
            (MIDDLE - 100.0, across_the_band),
            0.0,
            (0.0, 200.0),
            A_STEP * 3.0,
        );
        let across = game.log().written();
        game.image()
            .the_player_is_at(MIDDLE, FIELD.1 + FIELD.3 / 4.0);
        game.frames(1);
        game.one_frame_to_drain_the_log();
        assert!(
            game.log()
                .said_since(across, "in each box from frame never, 0, 0, 0, never"),
            "the boxes an arm lay across the band in were not the ones it lay across:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Fire below the player is not counted towards a box, whatever else is in it.**
///
/// What a box is counted for is deciding where to wait, and what makes a box a bad place to wait is the fire
/// that can still arrive. Below the player it has gone past: counting it fills the box the bot is standing in
/// with what has already missed, so the box reads worst just as it becomes safe and the box that is filling up
/// stands out only once the fire is level. Three hits in one run were pinned at an edge with every other box
/// reading `0.0` — `1.0, 24.7, 54.9, 0.0, 0.0` at the floor and `0.0, 0.0, 0.0, 0.0, 49.5` at the right wall.
#[test]
fn fire_below_the_player_is_not_counted_towards_a_box() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-fire-below-is-not-counted");
        // Where the rules want to be with nothing to collect, so a move of any kind means the counting sent it.
        game.image().the_player_is_at(MIDDLE, LOW);
        // A pile in its own box and below it, far enough down that none of it is a dodge of this frame.
        for slot in 0..60 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE + (slot % 4) as f32 * 4.0,
                LOW + 20.0 + (slot / 4) as f32 * 2.0,
                (0.0, 0.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left its box for fire that had already gone past it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Fire still on its way into the band is counted where it will be, not left out for not having arrived.**
///
/// A box is counted to decide where to *go*, and going there costs the frames of a crossing: two boxes at the
/// game's own step is most of a second. Counted where everything is at this instant, the answer describes the
/// field the bot is leaving rather than the one it will arrive in — so a wall that is a third of a second above
/// the band reads as an empty box to stand under, and by the time it reads as fire the crossing that would have
/// answered it no longer fits.
///
/// So a hazard counts towards the box it will be in when it comes into the window, as far ahead as
/// `bot::THE_FRAMES_AHEAD`. Below the player it still does not count at all — that is fire which has gone past,
/// and no amount of looking forward brings it back.
#[test]
fn fire_on_its_way_into_the_band_is_counted_where_it_will_be() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-fire-on-its-way-is-counted");
        // Where the rules want to be with nothing to collect, so a move of any kind means the counting sent it.
        game.image().the_player_is_at(MIDDLE, LOW);
        // A wall above the band the counting looks at, descending into the box the player is standing in. Far
        // enough up that the frames each move is followed for do not reach it, so nothing here is a dodge.
        for slot in 0..40 {
            game.image().a_bullet_of_size_is_at(
                slot,
                MIDDLE - 16.0 + (slot % 8) as f32 * 4.0,
                100.0 + (slot / 8) as f32 * 4.0,
                (0.0, 3.0),
                (4.0, 4.0),
            );
        }
        // And a little on its way into the two boxes left of the player, so that the box to wait in is to the
        // right of it whichever of the two rules picks it — the emptiest of the five, and the nearest of the
        // ones as empty as that.
        for slot in 40..48 {
            game.image().a_bullet_of_size_is_at(
                slot,
                FIELD.0 + FIELD.2 * [0.1, 0.3][slot % 2],
                100.0,
                (0.0, 3.0),
                (4.0, 4.0),
            );
        }
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot stood under a wall that had not arrived yet:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A border broken is a hit, and the respawn taking one away is not.**
///
/// The state a hit is noticed by is 2, the deathbomb window — and th07's border is deflected at 0x441bd0
/// instead: it reads the border off `[player+0xb7e6c]`, zeroes the byte at `+0x2cc` of what that points at and
/// nulls the pointer. So a run that survives on borders read as a run that was never hit, and every `0 hit(s)`
/// reported before this was reported without them.
///
/// **And a break is not any border going away.** Two places null that pointer: 0x441bf8, the deflection, and
/// 0x4417eb, the respawn tidying up after a death that has already been counted. The player's own state is what
/// tells them apart.
#[test]
fn a_border_broken_is_a_hit_and_a_respawns_is_not() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-border-broken-is-a-hit");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().the_player_has_a_border();
        game.frames(1);
        game.image().the_players_border_is_gone();
        game.frames(4);
        assert!(
            game.log()
                .said("autoplay: a border was broken, which is a hit"),
            "a broken border was not counted as a hit:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same pointer going away while the player is coming back from a death, which is the respawn's
        // own tidying and a hit already counted.
        let game = in_a_stage("th07-a-respawns-border-is-not-a-hit");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().the_player_has_a_border();
        game.frames(1);
        game.image().the_player_is_dying();
        game.image().the_players_border_is_gone();
        game.frames(4);
        assert!(
            !game.log().said("autoplay: a border was broken"),
            "the respawn taking a border away was counted as a hit:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A hit says the seconds before it and not only the frame of it.**
///
/// Which is the one thing every hit this session has needed and none of them had. Each was `nowhere open to it`
/// with every one of the nine hit *on the frame it was judged on* — a place lost before its frame began — and the
/// frame's own dump cannot say how the player came to be there. Four of them had the x not moving for thirty
/// frames at an edge, and that was only found by reading the raw `resume:` lines by hand afterwards. What the
/// player was doing, what it pressed and what the boxes read over the seconds before is the answer, and it has to
/// be kept while it happens because a hit takes it all off the screen.
#[test]
fn a_hit_says_the_seconds_before_it() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-says-the-seconds-before");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(4);
        game.image().the_player_is_at(MIDDLE + 40.0, LOW);
        game.frames(4);
        game.image().the_player_has_been_hit();
        game.frames(4);
        let log = game.log();
        assert!(
            log.said("autoplay: before it,"),
            "the hit said nothing of the frames before it:\n  {}",
            log.lines().join("\n  "),
        );
        // And the places are the ones the player was actually at, oldest first.
        assert!(
            log.said(&format!("at {MIDDLE:.0},")) && log.said(&format!("at {:.0},", MIDDLE + 40.0)),
            "the frames before the hit are not the places the player was at:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And the frames before a hit say where the rules wanted to be on each of them.**
///
/// Which is what the boxes beside them cannot settle. A run was hit at the floor with the box it had stood in
/// for three seconds climbing from `0` to `180` while two others held `10` — and the counting had said so from a
/// second and a half out. So either the rule never fired or something else was wanted more, and those are
/// different repairs: one is a threshold, the other is a boss or an item outranking it.
#[test]
fn the_frames_before_a_hit_say_where_the_rules_wanted_to_be() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-frames-before-say-the-wanting");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(4);
        game.image().the_player_has_been_hit();
        game.frames(4);
        assert!(
            game.log().said("wanting"),
            "the frames before the hit did not say where the rules wanted to be:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And what settled the place, so that the wanting can be read at all.**
///
/// A run was hit at the floor with the box it stood in climbing from `0` to `218` over three seconds, and the
/// frames before it said `wanting 328,384 for an item` — where `328` is a place an item could be and `384` is the
/// line the bot waits on, so the same pair reads as *the item's own place* or as *the emptiest box's middle with
/// nothing to collect*. Those are opposite findings: one is a rule outranked, the other is a rule that fired and
/// was not followed. So the place says which of the four settled it.
#[test]
fn the_frames_before_a_hit_say_what_settled_the_place() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-what-settled-the-place");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Nothing to collect and no boss, which is the middle of the field low.
        game.frames(4);
        game.image().the_player_has_been_hit();
        game.frames(4);
        assert!(
            game.log().said("wanting the middle"),
            "the frames before the hit did not say what settled the place:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The line the bot waits on leaves a whole look-ahead of downward steps above the floor.**
///
/// Which is what four hits in a session came to, and the arithmetic says why: the line was 48 off the bottom and
/// the frames looked at are twelve, so twelve downward steps of four put the player *at the clamp*. Every one of
/// those hits was a dodge on the way down arriving at the floor with the crossing it was in the middle of still to
/// do — a run went right to `222,432` with the emptiest box at `328` and was carried back to `118,432`, because at
/// the floor there is no downwards and three of the nine are the wall.
///
/// **A height kept as a habit and not a bound reached in trouble**: the room a dodge needs is room it has to have
/// before the trouble, since the frames looked at are the frames it has.
#[test]
fn the_line_the_bot_waits_on_leaves_a_look_ahead_of_room_below() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-wait-line-leaves-room");
        // Well below the line, so the answer is how far up it goes rather than which way.
        game.image()
            .the_player_is_at(MIDDLE, FIELD.1 + FIELD.3 - 8.0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            UP,
            "the bot did not come up off the floor:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And the line it is coming up to is far enough off the bottom that a whole look-ahead of downward steps
        // still lands above it. Read off the place the rules want rather than asserted about a constant, since
        // what matters is where the player ends up.
        // **More than a look-ahead and not exactly one.** At 48 off the bottom with twelve frames of a step of
        // four, the twelfth step down lands *on* the clamp — the room runs out on the last frame looked at, which
        // is the frame the trouble arrives on.
        let room = FIELD.1 + FIELD.3 - LOW;
        assert!(
            room > LOOK_AHEAD as f32 * A_STEP,
            "the wait line is {room} off the bottom, which a look-ahead of downward steps covers \
             exactly: {} leaves nothing after it",
            LOOK_AHEAD as f32 * A_STEP,
        );
    });
}

/// The frames each candidate move is followed for, as `bot::LOOK_AHEAD` has it.
const LOOK_AHEAD: usize = 12;

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
        // **More than the frame of the hit**, the line being one the frame loop writes in its own slack rather
        // than where it is worked out — see `detail!`.
        game.frames(4);
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

/// **What a hit names is everything whose box reaches the player, and not everything standing near it.**
///
/// A thing shaped like a beam is one enemy at the pivot it swings about, with a size as long as the field: its
/// middle is a hundred units up while its box lies across the player. Asked about middles, the dump of a hit
/// taken by exactly that said nothing was near it at all — which is the one answer that sends the reading
/// somewhere else, since the three arrays had already been ruled out by it.
#[test]
fn an_enemy_whose_box_reaches_the_player_is_named_near_a_hit_from_far_off() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-long-enemy-is-named-near-a-hit");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Far enough up that nothing about where it stands is near the player, and long enough down that the
        // box it hands the player's own test covers where the player stands.
        game.image()
            .an_enemy_of_size_is_at(3, MIDDLE, LOW - 100.0, (12.0, 600.0));
        // A frame for the rules to have a move of their own to report the hit against: the first one they are
        // asked for has no frame before it, and a hit on it says so instead.
        game.frames(1);
        game.image().the_player_has_been_hit();
        game.frames(4);
        assert!(
            game.log().said("near it, enemy 3"),
            "the hit did not name the enemy whose box lay across the player:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **What a hit names is measured off the same frame the things it names come from.**
///
/// The hit is noticed a frame or two after the collision — the state the game leaves at 2 is written inside its
/// own update, and orb reads before that update runs — so the player has moved by the time the arrays are
/// walked, and so has everything in them. Measuring the frame before's player against this frame's hazards puts
/// a step or two of error into every distance, which for a bullet crossing at four a frame is eight units of it:
/// a dump asked that way answered `nothing was near it at all` on a hit whose cause was standing in one of the
/// three arrays the whole time.
#[test]
fn what_a_hit_names_is_near_where_the_player_is_when_the_arrays_are_walked() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-names-what-is-near-it-now");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        // The player carried well away from where the frame before's move was decided, with the enemy at the
        // place it has got to rather than the place it came from.
        game.image().the_player_is_at(MIDDLE + 40.0, LOW);
        game.image()
            .an_enemy_of_size_is_at(3, MIDDLE + 40.0, LOW, (12.0, 12.0));
        game.image().the_player_has_been_hit();
        game.frames(4);
        assert!(
            game.log().said("near it, enemy 3"),
            "the hit did not name the enemy standing where the player had got to:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A hit leaves a picture of the frame in the game's own `snapshot` directory.**
///
/// Which is what the dump cannot be asked for: a hit that names nothing within reach of the player has
/// exhausted the three arrays, and whatever hit it was still drawn. The frame is a 32-bit BMP, so what a test
/// can hold it to is the header — that it says the screen's own size, and that its rows are the frame's.
///
/// **In a directory of its own and not beside the exe**, which is where the replays and the score file are: a
/// run that is hit twenty times puts twenty megabytes of frames somewhere, and the one place that is not is
/// among the files whoever is playing keeps.
#[test]
fn a_hit_leaves_a_picture_of_the_frame_in_the_snapshot_directory() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-a-hit-leaves-a-picture");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.frames(1);
        game.image().the_player_has_been_hit();
        game.frames(4);
        // **Named by the same stamp the log puts on its lines**, which is what makes the picture and the dump of
        // the hit it came out of findable from one another — and what keeps two runs' pictures apart, a count
        // starting over at nought every launch having put four hits from three practices under one name.
        let held_in = game.dir().join("snapshot");
        let written: Vec<_> = game
            .sim()
            .files()
            .paths()
            .into_iter()
            .filter(|path| path.starts_with(&held_in))
            .collect();
        let [picture] = written.as_slice() else {
            panic!("one picture is not what was written to {held_in:?}: {written:?}");
        };
        let name = picture.file_name().unwrap().to_string_lossy().to_string();
        let stamp = name
            .strip_prefix("hit-")
            .and_then(|rest| rest.strip_suffix(".bmp"))
            .unwrap_or_else(|| panic!("the picture is not named for a hit and a stamp: {name}"));
        assert!(
            !stamp.is_empty() && stamp.chars().all(|c| c.is_ascii_digit()),
            "the picture's name carries no stamp: {name}",
        );
        assert!(
            game.log().said(&name),
            "the log did not name the picture it wrote:\n  {}",
            game.log().lines().join("\n  "),
        );
        let held = game
            .sim()
            .files()
            .get(picture)
            .unwrap_or_else(|| panic!("nothing was written to {}", picture.display()));
        assert_eq!(&held[..2], b"BM", "what was written is not a BMP");
        assert_eq!(
            i32::from_le_bytes(held[18..22].try_into().unwrap()),
            SCREEN_ACROSS,
            "the picture is not as wide as the screen",
        );
        assert_eq!(
            i32::from_le_bytes(held[22..26].try_into().unwrap()),
            SCREEN_DOWN,
            "the picture is not as tall as the screen",
        );
        assert_eq!(
            held.len(),
            54 + (SCREEN_ACROSS * SCREEN_DOWN * 4) as usize,
            "the picture is not the size its own header says",
        );
    });
}

/// The screen the game shows through, which is the size a picture of one of its frames is.
const SCREEN_ACROSS: i32 = 640;
const SCREEN_DOWN: i32 = 480;

/// **A hit says what the model was holding near the player on the frame it chose the move.**
///
/// Which is the one reading nothing taken afterwards can stand in for. The arrays walked after a hit are a
/// frame or two on and the thing that did it has been taken off them — a bullet's own state goes to 5 the
/// moment the test comes back — so a dump of them says what is there *now*, and a hit whose cause was already
/// gone reads as a hit with nothing near it. What the model held is remembered instead of re-read, and a hit
/// with nothing in that list is a hazard the model never had.
#[test]
fn a_hit_says_what_the_model_held_near_the_player_when_it_chose() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-what-the-model-held");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Near enough to be held and far enough not to be a hit, so that what the list says is what the model
        // had rather than what it was steering away from.
        game.image()
            .a_bullet_is_at(12, MIDDLE + 10.0, LOW - 12.0, (0.0, 0.0));
        game.frames(1);
        // And gone from the arrays by the time the hit is read, the way the one that did it is.
        game.image().the_bullet_is_gone(12);
        game.image().the_player_has_been_hit();
        game.frames(4);
        assert!(
            game.log().said("held a bullet 12"),
            "the hit did not say the bullet the model was holding when it chose:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step aside is a way out and not another hit.** Which needs the room a move opens to be wider than what
/// counts as being hit: where the two are the same, every neighbour of a place a bullet crosses is a hit too,
/// so no move is open, and the choice falls through to the most room — which is the fleeing the whole of this
/// exists instead of, and fleeing a boss's pattern is deathbomb after deathbomb.
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

/// **And the move and the bullet are one frame's worth apart and not two.** One frame moves both, so the frame
/// a hit happens on is the one each has taken the same number of steps into — th07 and th06c alike advance a
/// bullet's own position and then hand *that* position to the test. A judgement that steps the bullet one
/// further than the move credits every move with a step of room the frame has not got, and the step a bullet
/// takes is whatever the pattern gave it: a stage 4 run of th06c was hit on a move its dump reported `2.6 of
/// room` for, by a bullet stepping `2.7` a frame.
///
/// **Which is a thing only a move can show**, standing still being the one choice the two readings agree on:
/// what the extra step hides is the player walking *into* something. So the bot is given an item to the left
/// and a bullet coming the other way along that line, one step of each apart — a frame from now the two are in
/// the same place, and the step towards the item is the step that is hit.
#[test]
fn the_bot_does_not_step_into_a_bullet_a_frame_from_where_it_is_going() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-pairs-the-frames");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image().an_item_is_at(700, POWER, 100.0, LOW);
        // Four of the player's own steps, which is a step long enough that a bullet stepped one time too many
        // is clear across the player rather than near it.
        let step = A_STEP * 4.0;
        game.image()
            .a_bullet_is_at(500, MIDDLE - A_STEP - step, LOW, (step, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & LEFT,
            0,
            "the bot stepped into a bullet a frame from where that step lands it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a bullet it has room from stops nothing.** A band wide enough to take in most of what is on the screen
/// is a bot that spends every frame leaving and never collects anything: the screen of a th07 stage is never
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

/// **A boss is shot at the focused step**, which is where th07's shots do their damage — and only where there is
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
        // threat, and it is not what a hit is either: the same twelve units are inside a large bullet's own half
        // and outside a small one's, so a slowing taken off by a distance is one taken off for one bullet and not the
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

/// A dialogue is skipped, which is th07's own Ctrl: a bot that sat through every one of them would spend the
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
/// still set — which is the launch that walked th07's own title menu onto Quit and took it: nothing had been
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

/// Nor while the run is paused, which is th07's own pause menu — the same shot, and the same d-pad.
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

/// **A bullet as wide as th07's widest has to be moved for before it is a step away.**
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
/// tighter of the two axes — and a distance between middles is not that: a small bullet six units away has two
/// units of room and a large bullet seventeen away has one, and the middles put them the other way round. Which
/// is not only a number in a log: `nearest was a bullet …` is what a hit is read against, and a line naming a
/// bullet the player was drifting away from is a line that sends whoever reads it to the wrong part of the
/// model. One did:
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
        // **More than the frame of the hit**, the line being one the frame loop writes in its own slack rather
        // than where it is worked out — see `detail!`.
        game.frames(4);
        assert!(
            game.log().said("with 3.0 of room")
                && game.log().said(&format!(
                    "nearest was a bullet at {big:.1},{LOW:.1} stepping 0.0,0.0, half of it \
                     {:.1},{:.1}",
                    WIDEST_BULLET / 2.0,
                    WIDEST_BULLET / 2.0,
                )),
            "the hit named the bullet whose middle was nearest rather than the one whose box was:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A third of the field with the fire clearly piled into it is not where the bot waits.**
///
/// Which is the one thing the moves themselves cannot say. Each of the nine asks whether it is hit over the
/// frames looked at, so a bot can stand in the thick of a pattern for as long as the next twelve frames happen
/// to be survivable and be somewhere with nothing open the moment they are not — and the hits say so: `nowhere
/// open to it` with a model holding 123 bullets and nine bodies, which is not a hazard missed but a place
/// chosen.
///
/// So the field is cut into three down its width, the bullets in each are counted, and where the rules want to
/// be is moved out of a third that is *clearly* the thickest. **Clearly, and by a factor**: an ordinary
/// imbalance is not a reason to leave a fight, and a factor means the same thing at thirty bullets as at three
/// hundred where a difference would not.
#[test]
fn a_box_of_the_field_with_the_fire_piled_into_it_is_not_waited_in() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-keeps-out-of-the-thick");
        game.image().the_player_is_at(MIDDLE, LOW);

        // Nothing anywhere near the player, so no move of the nine is shut and the only thing that can move it
        // is where it wants to be.
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot left the middle of an empty field:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And a pile of them in the middle third, high up the field and going nowhere.
        for slot in 0..30 {
            game.image()
                .a_bullet_is_at(slot, MIDDLE, LOW - 40.0, (0.0, 0.0));
        }
        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert!(
            pressed == LEFT || pressed == RIGHT,
            "the bot waited in the thickest third of the field, pressing {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A point item is not worth walking into the thick of it for, and a big power item is.**
///
/// Which is where the two rules meet: the thickest third is not somewhere to wait, and an item is somewhere to
/// go — so one of them has to give, and what decides is what the item is worth. Being crushed costs a bomb at
/// best and the run at worst, and a point item is worth neither; a 1up, a bomb, or the eight steps of power the
/// big one carries are worth the crossing.
#[test]
fn the_thick_of_it_is_crossed_for_a_big_power_item_and_not_for_a_point() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-weighs-the-item-against-the-fire");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The fire piled into the leftmost box, high up and going nowhere, and the item in among it.
        for slot in 0..30 {
            game.image()
                .a_bullet_is_at(slot, FIELD.0 + 30.0, LOW - 40.0, (0.0, 0.0));
        }
        // And a pile in each of the other boxes but the one right of the middle, so that it is the only box
        // left to wait in — which is what makes the two halves of this tell apart by the way the bot goes, and
        // it has to be the only one: the box the player stands in is as good as any other that is no fuller,
        // and standing still tells them apart from nothing. Not the rightmost, an edge being counted no
        // emptier than the box beside it.
        for slot in 30..48 {
            game.image().a_bullet_is_at(
                slot,
                FIELD.0 + FIELD.2 * [0.25, 0.45, 0.9][slot % 3],
                LOW - 40.0,
                (0.0, 0.0),
            );
        }
        game.image().an_item_is_at(700, POINT, FIELD.0 + 30.0, LOW);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot did not leave the point item in the thick of it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same place, worth crossing for.
        game.image().the_item_is_gone(700);
        game.image()
            .an_item_is_at(701, BIG_POWER, FIELD.0 + 60.0, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & LEFT,
            LEFT,
            "the bot left the big power item where it lay:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Of two open moves the same distance from what the rules want, the emptier side is taken.**
///
/// Which costs nothing to know and nothing to decide: the room each move has is already measured, and this only
/// says what to do where the wanting cannot tell two moves apart — which is every symmetric pair, and so every
/// frame the bot is standing where it wanted to be. Today that goes to whichever comes first in the list of
/// moves, a fixed bias towards one side of the screen with nothing behind it.
///
/// **The lowest priority there is**, deliberately. Ranked above the wanting it makes a bot that steps whenever a
/// neighbour is a shade freer and never arrives — five of these tests said so at once — and as a threshold on
/// how cramped is too cramped the number has to be guessed. As a tie-break it can only replace an arbitrary
/// choice with a measured one.
#[test]
fn of_two_open_moves_the_same_way_off_the_emptier_side_is_taken() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-takes-the-emptier-side");
        game.image().the_player_is_at(MIDDLE, LOW);
        // One coming down the player's own column, late enough that standing still is what it shuts and the
        // ways aside stay open — so the choice is between moves, all of them the same way off the middle.
        game.image()
            .a_bullet_is_at(0, MIDDLE, LOW - A_STEP * 10.0, (0.0, A_STEP));
        // And one sitting off to the left, near enough to take the room off that side and not to shut it.
        game.image()
            .a_bullet_is_at(1, MIDDLE - A_STEP * 4.0, LOW, (0.0, 0.0));

        game.frames(1);
        let pressed = game.image().the_word_the_run_acts_on() & PAD;
        assert_eq!(
            pressed & RIGHT,
            RIGHT,
            "the bot did not take the emptier of two sides it wanted equally, pressing \
             {pressed:#05x}:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed & LEFT,
            0,
            "the bot went towards the bullet it had room from, pressing {pressed:#05x}:\n  {}",
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
/// enough ahead to leave a large bullet they are not. A launch that looked twelve frames out and still ranked by the
/// count was hit nine times before stage 3's boss, where the launch before it had reached stage 4 without a scratch.
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

/// **A bullet that did not go where its own step says is one to be afraid of rather than to project.**
///
/// 0x425ccd adds `+0xb98` to a bullet's place and that is the whole of an ordinary one's motion, so where it
/// will be next frame is known exactly. **Some are not moved that way**: stage 1's last card and stage 6's
/// turning one are beams whose pieces a boss's script moves, and no reading of the bullet says where a script
/// will put it. Those are what kept hitting a bot that had them in its model — `13.1 of room` and `3.2`, the projection
/// pointing one way while the piece went another.
///
/// So the disagreement is the signal: what a bullet actually moved last frame against what its step claims. Past
/// that there is nothing to project, and what is left is to keep away from everywhere it could reach — its box
/// grown by its own speed for every frame looked at.
#[test]
fn a_bullet_that_ignores_its_own_step_is_feared_rather_than_projected() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-fears-a-scripted-bullet");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Well clear, and claiming to be going nowhere.
        game.image()
            .a_bullet_is_at(0, MIDDLE + A_STEP * 10.0, LOW, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for a bullet ten steps away and going nowhere:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the same bullet somewhere its step could not have taken it, which is a script moving it.
        game.image()
            .a_bullet_is_at(0, MIDDLE + A_STEP * 7.0, LOW, (0.0, 0.0));
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot held still beside a bullet no step of its own explains:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet is moved for whatever state it is in but nothing.**
///
/// Which is a rule the exe argues against and two launches settle. The dispatch at 0x425b82 sends a bullet
/// through one of five cases on its state and the two calls that hand one to the player's own tests are both
/// inside the first, so leaving the other four out reads as safe — and it is not: the run that left them out
/// cleared the game with nine hits where the run before it had four, eight of them on stage 6's boss reading
/// between six and fourteen units of room, which is what the log says when the thing that hit was never in the
/// model.
///
/// What the reading missed is that the state is a *kind* of bullet and not a phase one passes through:
/// 0x423a5d, 0x423e66, 0x423f83 and 0x4240e7 write 1, 2, 3 and 4 from four separate places in the spawn, and
/// only one case, at 0x425ff8, ever turns into the first. So a bullet in the other three is a bullet that stays
/// there.
#[test]
fn a_bullet_is_moved_for_whatever_state_it_is_in() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-moves-for-every-bullet");
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(0, MIDDLE + A_STEP, LOW, (0.0, 0.0));

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood on a bullet in the state the collision was found in:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_bullet_is_in_another_state(0);
        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood on a bullet for being in one of the other four states:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_bullet_is_gone(0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for a slot holding no bullet at all:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy with no body to run into is not moved for at all.**
///
/// 0x420e86 and 0x420e9f gate the enemy's own collision on the low two bits of `+0x2e29`, and 0x420ebc — the
/// only place an enemy's position and size reach the player's box test — is past both. So an enemy with either
/// bit down cannot touch the player however near it stands.
///
/// **Which th07's stage 6 boss makes the difference between a clear and a cornering.** It fires an aimed shot off an
/// enemy its script keeps at the player: invisible, eight units across, and drifting a fraction of a unit a
/// frame. A bot that read every living enemy as solid had a hazard it could not get away from, and three hits
/// in the one chapter said `nowhere open to it` — all nine moves shut by a thing that was never going to touch
/// it.
#[test]
fn an_enemy_with_no_body_is_not_moved_for() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-ignores-a-firing-point");
        game.image().the_player_is_at(MIDDLE, LOW);
        // Right beside the player and wide enough to be a hit if it were one.
        game.image()
            .an_enemy_of_size_is_at(0, MIDDLE + A_STEP, LOW, (24.0, 24.0));

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood inside an enemy's body:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_enemy_has_no_body(0);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot moved for an enemy the game would not have let touch it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy is moved for by where it is going and not only by where it stands.**
///
/// Nothing in an enemy's own update is a step a read could take — its script moves it — so the only way to
/// know is to remember where it was last frame, which is the way a laser's turn is known. A body taken for a
/// still thing is a body the model says there is room beside while it closes: a launch was hit beside stage 2's
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
/// much as touches them. th07's stage 4 is where that costs a run: three bosses share the stage, and a launch
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

/// **And a boss against the wall is followed only as far as a dodge has room.**
///
/// A corner is where a dodge runs out of moves: three of the nine lead into the wall, where the clamp makes them
/// the move the player is already making. th07's stage 4 is where that gets paid — three bosses share the stage
/// and one of them sits at the left edge — and a run that followed it was hit at `27.8,432.0` with **nowhere
/// open to it**, which is the log's way of saying all nine were shut.
///
/// Only the x *wanted* is held off the edge: a dodge still goes wherever the field allows, the same way
/// [`the_bot_stays_in_the_bottom_third_of_the_field`] bounds where the bot waits and not where it may step.
#[test]
fn a_boss_against_the_wall_is_followed_only_as_far_as_a_dodge_has_room() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-keeps-off-the-wall");
        game.image().a_boss_is_fighting(1000, 0);
        game.image().the_boss_is_at(FIELD.0 + 8.0, 100.0);

        // Standing where it will wait, it has nowhere further to go.
        game.image()
            .the_player_is_at(FIELD.0 + IN_FROM_THE_SIDE, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot went on towards the wall for a boss standing against it:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And out in the field it still closes on the fight.
        game.image()
            .the_player_is_at(FIELD.0 + IN_FROM_THE_SIDE + 100.0, LOW);
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot gave up closing on a boss it had room to reach:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A boss slot leading off the field is passed over for the next one, not given up on.**
///
/// th07's stage 4 is three bosses in eight slots, and the ones the fight is not with wait off the side of the
/// field — alive, so no aliveness test skips them, and outside it, so their x is nothing to stand under. A rule
/// that took the first alive slot and then found it unusable stood in the middle of the field for the whole
/// fight: **a launch did exactly that**, and the middle is the answer for having no boss at all.
#[test]
fn a_boss_slot_leading_off_the_field_is_passed_over_for_the_next() {
    in_its_own_process(|| {
        let game = in_a_stage("th07-the-bot-passes-over-a-boss-off-the-field");
        game.image().the_player_is_at(MIDDLE, LOW);
        // The first slot alive and waiting off the left of the field, the second where the fight is.
        game.image().a_boss_slot_leads_to(0, 0);
        game.image().an_enemy_is_at(0, FIELD.0 - 50.0, 100.0);
        game.image().a_boss_slot_leads_to(1, 1);
        game.image().an_enemy_is_at(1, 300.0, 100.0);

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            RIGHT,
            "the bot stood in the middle rather than under the boss it could reach:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And with every slot leading off the field there is no boss to stand under.** The x of something the player
/// cannot get under is an x it walks into a wall for, and a wall is where a dodge runs out of moves — so the
/// middle is a better place to be than under a boss that is not there to be stood under.
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

/// **The focused step is for the fight and not for the collecting**: the same key that puts the damage up halves the
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

/// **A chapter of a run being played**, which is what a chapter drawn to be hit in needs: the draw is made for
/// the chapter the run is in, and a stage that has taken none is a run with no chapter to be drawn for.
///
/// The whole walk in — the mode question answered at the title, the stage built, its first chapter taken — and
/// then the field and the player put where [`in_a_stage`] puts them, so that a move is read the same way in
/// both.
fn in_a_chapter(name: &str, recipe: &str) -> Box<Fake> {
    let game = playing_to(name, recipe);
    game.in_a_pointdevice_run();
    game.image().the_player_is_playing();
    game.image()
        .the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
    game.image().the_players_box_is((A_STEP, A_STEP));
    game.image()
        .the_play_field_is(FIELD.0, FIELD.1, FIELD.2, FIELD.3);
    game.image().no_boss_is_fighting();
    game.image().is_not_paused();
    game
}

/// A chance of one, which is every chapter drawn to be hit in — what a test about the hits asks for, the draw
/// itself being read where the numbers are.
const EVERY_CHAPTER: &str = "- take_hits: { chance: 1.0 }\n";

/// **A recipe that names the item to answer the retry question with gets to answer it.**
///
/// orb's own offer to retry is taken wherever it comes up, a death landing in the middle of a step — and what
/// takes it is a tap of decide, which is the item the cursor is on. That is the first of them, so a recipe
/// naming any other was answered before its own step saw the question and then sat out its bound: a run asked
/// to start over could be sent back to the chapter it was in and nowhere else.
///
/// **Which is what a check of a stage's own first chapter needs**, the stage being one of the items: that
/// chapter is then a launch of a minute away, where a walk to it is the whole run played again — half an hour
/// of autoplay for a file the next launch overwrites.
///
/// **The item is the stage's place in the list as the list stands**, and what stands in it depends on the
/// chapter: a run at a stage's own first chapter has nothing behind that chapter, so the item that lists what
/// is behind it is left out and the stage is the second. A recipe naming the third here asks for the title.
///
/// **And the front end pressed through by the recipe's own first step**, which is what the copies driven by
/// these recipes do: a launch playing to one answers every read of the game's own keyboard, so the presses the
/// walk-in wants have to be the bot's — a step that waits presses nothing, and the walk-in then waits for a
/// decide nobody makes.
#[test]
fn a_recipe_of_th07_answers_the_retry_question_with_the_item_it_names() {
    in_its_own_process(|| {
        let game = in_a_chapter(
            "th07-the-bot-answers-the-retry",
            "- press: [decide]\n  frames: 10\n  until: { stage: 1, chapter: 1 }\n  \
             give_up_after: 1800\n\
             - start_over: true\n\
             - answer: { question: retry, item: 1 }\n",
        );
        let log = game.log();
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

/// **A chapter drawn to be hit in is not dodged in**: the move is the one towards where the rules want to be
/// and the hazards decide none of it, which is what somebody who has stopped paying attention plays like.
///
/// Read against [`the_bot_steps_around_a_bullet_rather_than_fleeing_it`], which is the same bullet coming down
/// onto the same player: there the bot steps out from under it, and here it stands where it is and takes it.
#[test]
fn a_chapter_drawn_to_be_hit_in_is_not_dodged_in() {
    in_its_own_process(|| {
        let game = in_a_chapter("th07-the-bot-takes-a-hit", EVERY_CHAPTER);
        game.image().the_player_is_at(MIDDLE, LOW);
        // Coming down onto where the player stands, and where the player stands is where the rules want to be:
        // so a move of any kind is a move made about the bullet.
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - 24.0, (0.0, 8.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot dodged a bullet in a chapter drawn to be hit in:\n  {}",
            game.log().lines().join("\n  "),
        );
        // The frame that says it is the frame after the one that decided — see
        // `Launched::one_frame_to_drain_the_log`.
        game.one_frame_to_drain_the_log();
        assert!(
            game.log()
                .said("autoplay: stage 1 chapter 1 is one to be hit in"),
            "the chapter was not said to be one to be hit in:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And not bombed out of either**, which is the whole of what the drawing is for: a deathbomb is the death
/// taken back, so a chapter played to be died in and bombed is a chapter nothing came of.
#[test]
fn a_chapter_drawn_to_be_hit_in_is_not_bombed_out_of() {
    in_its_own_process(|| {
        let game = in_a_chapter("th07-the-bot-does-not-death-bomb", EVERY_CHAPTER);
        game.image().the_player_is_at(MIDDLE, LOW);

        game.image().the_player_has_been_hit();
        // The same frames [`the_bot_bombs_the_frame_the_player_is_hit`] gives the bomb to arrive in.
        game.frames(4);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            0,
            "the bot took back a death its chapter was drawn for:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And nothing having hit it, it goes to the nearest thing that can.**
///
/// Which is what makes the chance a chance of being *hit* rather than of playing carelessly: a chapter whose
/// fire never reaches the place the rules want to be would go by unhit, and a chapter is short — the 114 two
/// launches played through had a median of 237 frames.
#[test]
fn nothing_having_hit_it_the_bot_goes_to_the_nearest_thing_that_can() {
    in_its_own_process(|| {
        let game = in_a_chapter("th07-the-bot-goes-to-a-bullet", EVERY_CHAPTER);
        game.image().the_player_is_at(MIDDLE, LOW);
        // Standing still, well off to the left: nothing about it reaches the player, so nothing about it is a
        // thing to dodge or to be hit by where it is.
        game.image().a_bullet_is_at(500, 100.0, LOW, (0.0, 0.0));

        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot went for the bullet on the chapter's first frames:\n  {}",
            game.log().lines().join("\n  "),
        );

        // A second of them, which is how long a chapter drawn to be hit in is played before it goes looking.
        game.frames(60);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & PAD,
            LEFT,
            "the bot never went to the one thing that could hit it:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a chance of nought is the rules dodging everything**, which is what a recipe that says nothing gets:
/// the step is what turns this on, and turning it on with no chance is turning it off.
#[test]
fn a_chance_of_nothing_is_a_chapter_played_as_any_other() {
    in_its_own_process(|| {
        let game = in_a_chapter(
            "th07-the-bot-takes-no-hits",
            "- take_hits: { chance: 0.0 }\n",
        );
        game.image().the_player_is_at(MIDDLE, LOW);
        game.image()
            .a_bullet_is_at(500, MIDDLE, LOW - 24.0, (0.0, 8.0));

        game.frames(1);
        assert_ne!(
            game.image().the_word_the_run_acts_on() & PAD,
            0,
            "the bot stood under a bullet in a chapter drawn for nothing:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a death can be answered by going back to the title instead of by taking the chapter again**, which is
/// the other thing somebody who has just died does — and the one thing that leaves a run in a file for the next
/// launch to pick up.
///
/// **Asked for rather than pressed for.** orb's own offer keeps its own cursor and hands a bot nothing to read
/// it by, and how many items are on it is how many chapters the stage has kept behind the one that was lost —
/// so what answers this is `runtime::give_the_run_up_at_the_retry`, and the log names the ask rather than a
/// hand.
#[test]
fn a_death_drawn_to_be_given_up_at_goes_back_to_the_title() {
    in_its_own_process(|| {
        let game = in_a_chapter(
            "th07-the-bot-gives-the-run-up",
            "- take_hits: { chance: 1.0, gives_up: 1.0 }\n",
        );
        let log = game.log();

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.frames_until("the run given up", 240, || {
            log.said("retry: the run given up on the recipe's own ask")
        });
        assert_eq!(
            game.image().the_scene_wanted(),
            game.image().scene_of_the_front_end(),
            "the run was not put on its way to the title:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **The chance is lowered each time the chapter is retried**, so that a run gets past the chapter it died in.
///
/// At the same chance every attempt a chapter is retried until the fire happens to miss, which is an afternoon
/// spent on one chapter. What counts a retry is orb's own offer having been up and being gone: a retry puts the
/// chapter it was asked from back, so its number is the same on either side of the death.
#[test]
fn the_chance_is_lowered_each_time_the_chapter_is_retried() {
    in_its_own_process(|| {
        let game = in_a_chapter(
            "th07-the-bots-chance-falls-each-retry",
            "- take_hits: { chance: 1.0, times_each_retry: 0.5 }\n",
        );
        let log = game.log();
        game.frames_until("the chapter's first attempt drawn for", 8, || {
            log.said("autoplay: stage 1 chapter 1 is one to be hit in, its chance being 1.000 after 0 retr(ies)")
        });

        // The death, and the offer answered by the bot itself: its word goes to orb's own menus as a pad's
        // buttons do, which is what lets a launch nobody is watching carry on past a death.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.frames_until("the chapter taken again", 240, || {
            log.said("retry: the chapter again")
        });

        // **The chance and not which way the draw went**, which is what this test is about: the attempt after
        // a retry is drawn at half of what the one before it was, and a chance of a half is a chance either
        // way.
        game.frames(60);
        assert!(
            log.said("its chance being 0.500 after 1 retr(ies)"),
            "the attempt after the retry was not drawn at half the chance:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
