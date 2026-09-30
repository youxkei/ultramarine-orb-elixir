//! **What a stage of th10 leaves its player as, and what a hit does to it: the states orb reads as a run in the
//! middle of something.**
//!
//! Read off the exe — `asm.md`'s *The player* has the fields:
//!
//! - **A run's player is made playing with nothing to hit it for 120 frames**: the player's making registers its
//!   jobs and puts `0x78` under the count at `+0x4310` (0x424d14), and a stage's start puts the state at `+0x458`
//!   at 1 (0x424d90) without touching that count — so a run's first stage has the player playing from its first
//!   frame, with the count running down one an update (0x425e46 through 0x425eaf) and both of the player's tests
//!   declining a hit while it is above nought (0x426756, 0x426950).
//! - **A hit is state 4** (0x426cf0), eight updates a bomb could take it back in (0x425a96), and then state 2: the
//!   life taken and 180 under the count (0x426ac4) for thirty updates (0x425b9d), after which the player is put
//!   back at the bottom of the field in state 0 with 280 under the count (0x425ceb through 0x425d1e) and walked up
//!   from there for sixty (0x425751 through 0x42577b, 0x425902 through 0x42590f).
//! - **And only state 1 reads the run's word to move** (0x4250b0 from the switch at 0x425739), so a direction held
//!   through the hit, the death and the coming back moves nothing until the player plays again.
//!
//! **What orb reads off that is `State::unsettled`**, any state but 1, which is what keeps a chapter from being
//! taken while a run is in the middle of a death. **The chapters are off here**, so that the hit is the game's to
//! carry out rather than the retry menu's to answer.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The player's four states, written out here rather than taken from `Th10`'s own: a test that read them off the
/// code under test would agree with it whatever either said.
const COMING_BACK_IN: i32 = 0;
const PLAYING: i32 = 1;
const DYING: i32 = 2;
const HIT: i32 = 4;

/// What a run's player is made with under the count, what a death puts there and what coming back in puts there.
const A_RUN_BEGINS_UNHITTABLE_FOR: i32 = 0x78;
const A_DEATH_LEAVES_IT_UNHITTABLE_FOR: i32 = 180;
const COMING_BACK_LEAVES_IT_UNHITTABLE_FOR: i32 = 280;

/// How many updates each state of a hit is seen in, counted from the one after it began.
const A_HIT_IS_SEEN_FOR: u32 = 8;
const DYING_IS_SEEN_FOR: u32 = 30;
const COMING_BACK_IN_IS_SEEN_FOR: u32 = 60;

/// Where coming back in starts and ends, in hundredths: the middle of the field across, and `480.0` then `400.0`
/// down (0x425751 through 0x42577b).
const COMING_BACK_IN_FROM: (i32, i32) = (0, 48_000);
const COMING_BACK_IN_TO: (i32, i32) = (0, 40_000);

/// How many frames a stage's build is given.
const A_BUILD: u32 = 600;

/// A run placed in its first stage and past its first frame — the build over and the stage begun, which is
/// where the player's state is put at 1 — with nothing fired at the player.
fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.chapters = false,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames_until("the stage's first frames", A_BUILD, || {
        game.state().in_run && game.state().stage_frames > 0
    });
    game
}

/// How many frames in a row the player's state stays `state`, from the next frame on, and the state it is in on
/// the first frame it is not.
fn frames_in(game: &Fake, state: i32, at_most: u32) -> (u32, i32) {
    for frame in 0..at_most {
        game.frame();
        let (now, _) = game.image().what_the_player_is_doing();
        if now != state {
            return (frame, now);
        }
    }
    panic!("the player stood in state {state} for more than {at_most} frames");
}

/// A run's first stage begins with its player playing and nothing to hit it for 120 updates — and orb reads
/// that as a run standing where a chapter can be taken.
#[test]
fn a_run_of_th10_begins_with_its_player_playing_and_unhittable_for_120_updates() {
    in_its_own_process(|| {
        let game = in_a_run("th10-the-players-first-frames");
        let (state, _) = game.image().what_the_player_is_doing();
        let unhittable = game.image().the_frames_nothing_hits_the_player_for();
        assert_eq!(
            state, PLAYING,
            "the stage did not begin with its player playing"
        );
        assert!(
            (A_RUN_BEGINS_UNHITTABLE_FOR - 2..=A_RUN_BEGINS_UNHITTABLE_FOR).contains(&unhittable),
            "the run's player was not made with nothing to hit it for 120 updates: {unhittable}",
        );
        assert!(
            !game.state().unsettled,
            "a player playing reads as a run in the middle of something",
        );
        game.frames(unhittable as u32);
        assert_eq!(
            game.image().the_frames_nothing_hits_the_player_for(),
            0,
            "the count under the player did not run down one an update",
        );
    });
}

/// **A hit, in the three states the game takes it through**: eight updates of the hit, the life taken and thirty
/// of dying, and the player put back at the bottom coming back in for sixty — moving nowhere on a direction held
/// all the way, and read by orb as unsettled through every one of them.
#[test]
fn a_hit_of_th10_is_the_hit_then_the_player_dying_then_coming_back_in() {
    in_its_own_process(|| {
        let game = in_a_run("th10-the-players-hit");
        let image = game.image();
        let (_, _, lives) = image.the_runs_numbers();
        let deaths = game.state().deaths;
        let hit_at = (2_000, 30_000);
        image.the_player_is_at_hundredths(hit_at.0, hit_at.1);
        game.the_player_is_hit();
        game.keyboard().set(keys::UP, true);

        // The hit, which moves the player nowhere on the held direction.
        let (frames, then) = frames_in(&game, HIT, A_HIT_IS_SEEN_FOR * 2);
        assert_eq!(
            (frames, then),
            (A_HIT_IS_SEEN_FOR, DYING),
            "the hit did not stand for the eight updates a bomb could take it back in",
        );
        assert_eq!(
            image.where_the_player_is_in_hundredths(),
            hit_at,
            "the player moved while it was hit",
        );
        let state = game.state();
        assert_eq!(
            (state.lives, state.deaths),
            (lives as i8 - 1, deaths + 1),
            "the hit run out did not take the life, or orb did not count it once",
        );
        assert!(
            state.unsettled,
            "a player dying reads as a run standing still"
        );
        assert!(
            (A_DEATH_LEAVES_IT_UNHITTABLE_FOR - 1..=A_DEATH_LEAVES_IT_UNHITTABLE_FOR)
                .contains(&image.the_frames_nothing_hits_the_player_for()),
            "the death did not leave nothing to hit the player for 180 updates",
        );

        // Dying, which moves it nowhere either.
        let (frames, then) = frames_in(&game, DYING, DYING_IS_SEEN_FOR * 2);
        assert_eq!(
            (frames + 1, then),
            (DYING_IS_SEEN_FOR, COMING_BACK_IN),
            "the player did not die for thirty updates before coming back in",
        );
        assert!(
            (COMING_BACK_LEAVES_IT_UNHITTABLE_FOR - 1..=COMING_BACK_LEAVES_IT_UNHITTABLE_FOR)
                .contains(&image.the_frames_nothing_hits_the_player_for()),
            "coming back in did not leave nothing to hit the player for 280 updates",
        );
        let (x, y) = image.where_the_player_is_in_hundredths();
        assert!(
            x == COMING_BACK_IN_FROM.0 && y <= COMING_BACK_IN_FROM.1 && y > COMING_BACK_IN_TO.1,
            "the player did not come back in from the bottom of the field: ({x}, {y})",
        );
        assert!(
            game.state().unsettled,
            "a player coming back in reads as a run standing still"
        );

        // Coming back in, walked up by the game and not by the held direction, and playing again at its end.
        let (frames, then) = frames_in(&game, COMING_BACK_IN, COMING_BACK_IN_IS_SEEN_FOR * 2);
        assert_eq!(
            (frames + 1, then),
            (COMING_BACK_IN_IS_SEEN_FOR, PLAYING),
            "the player did not come back in for sixty updates before playing again",
        );
        assert!(
            !game.state().unsettled,
            "a player playing again reads as a run in the middle of something",
        );
        // **And moves again on the update it plays**, the state its word is read in again.
        game.frame();
        assert!(
            image.where_the_player_is_in_hundredths().1 < COMING_BACK_IN_TO.1,
            "the player did not move on the held direction once it played again",
        );
        game.keyboard().set(keys::UP, false);
    });
}
