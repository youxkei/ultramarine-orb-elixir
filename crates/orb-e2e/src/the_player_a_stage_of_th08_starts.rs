//! **What a stage of th08 leaves its player as, and what a hit does to it: the states nothing can kill it in.**
//!
//! Read off the exe — `asm.md`'s *The player* has every address:
//!
//! - **A stage's build registers the player on its own thread** (0x44c230 on both of the build's paths, the
//!   added callback at once at 0x43c8a8), which leaves the state byte at `[player+0x0]` at **1** and the count
//!   under it at `[player+0xe2af4+0x8]` at **120** (0x44d886 through 0x44d8a7). While the build is held the run
//!   manager ends the walk before the player's own job at priority 9 (0x439fe2), so the player stands in state 1
//!   for as long as the build does.
//! - **Its first update after that puts it in state 3 with 0xf0 under it** — 0x44d180 at a count of thirty or
//!   more (0x44d24a, 0x44d252, 0x44d29f) — and state 3 counts down one an update (0x44d35c) and goes back to 0 on
//!   the update it reaches nothing (0x44d39f). So 240 updates of every stage are ones the box tests decline,
//!   each of them asking the state byte first (0x44a333, 0x44a445, 0x44a8f2).
//! - **A hit is state 2 with the count from nothing** (0x44ab84); at thirty 0x44cbf0 takes the life (0x44d100)
//!   and puts the player back at the start in state 1 with the count from nothing again (0x44d012 through
//!   0x44d07a), which thirty later is state 3 as above.
//! - **And states 1 and 2 move nothing** — 0x44c4b5 through 0x44c4cb step over the whole of the player's own
//!   input, the moves at 0x44aee8 with it.
//!
//! **Read off the exe and not off this tree**, because the laid-out game is both the writer and the reader of
//! the memory orb is driven through: a player back to playing the frame its life went is one every e2e test here
//! would agree with, and orb's `unsettled` — any state but 0 — would never have stood for a frame of one.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The player's four states, written out here rather than taken from `Th08`'s own, which keeps two of them: a
/// test that read them off the code under test would agree with it whatever either said.
const PLAYING: u8 = 0;
const COMING_IN: u8 = 1;
const HIT: u8 = 2;
const UNTOUCHABLE: u8 = 3;

/// What a stage's build leaves under the player, and what its coming in turns into.
const THE_BUILD_LEAVES: i32 = 120;
const UNTOUCHABLE_FOR: i32 = 0xf0;

/// How many updates, counted from the one after it, each state of a hit is seen in: thirty of the hit and
/// thirty of coming in, each until its count has reached thirty, and one fewer than the count of the state that
/// runs down, whose first update already takes one off.
const A_HIT_IS_SEEN_FOR: u32 = 30;
const COMING_IN_IS_SEEN_FOR: u32 = 30;
const UNTOUCHABLE_IS_SEEN_FOR: u32 = UNTOUCHABLE_FOR as u32 - 1;

/// The stage the run walks into, orb's own 0-based number.
const THE_NEXT_STAGE: i32 = 1;

/// How many frames a stage's build is given.
const A_BUILD: u32 = 600;

/// A run placed in its first stage and playing, with nothing fired at the player.
fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames_until("the run", 60, || game.state().in_run);
    game
}

/// The state byte and the count under it, as the player's own update left them.
fn the_player(game: &Fake) -> (u8, i32) {
    (
        game.image().what_the_player_is_doing(),
        game.image().the_frames_the_hit_has_counted(),
    )
}

/// How many frames in a row the player's state stays `state`, from the next frame on, and the state it is in
/// on the first frame it is not.
fn frames_in(game: &Fake, state: u8, at_most: u32) -> (u32, u8) {
    for frame in 0..at_most {
        game.frame();
        let now = game.image().what_the_player_is_doing();
        if now != state {
            return (frame, now);
        }
    }
    panic!("the player stood in state {state} for more than {at_most} frames");
}

/// A stage's build leaves the player coming in with the register's count under it — and orb reads that as a
/// run in the middle of something.
#[test]
fn a_stage_of_th08_is_built_with_its_player_coming_in() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-players-build");
        game.moves_on_to_the_stage(THE_NEXT_STAGE);
        game.frames_until("the stage's build", A_BUILD, || {
            game.image().the_loader_is_running()
        });
        game.frame();
        assert_eq!(
            the_player(&game),
            (COMING_IN, THE_BUILD_LEAVES),
            "the stage's build did not leave the player as its own register leaves them",
        );
        assert!(
            game.state().unsettled,
            "a player coming in reads as a run standing somewhere a chapter can be taken",
        );
    });
}

/// And the first update after it is 240 of nothing reaching the player, which is every stage's own.
#[test]
fn a_player_of_th08_comes_out_of_a_stages_build_untouchable_for_240_updates() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-players-first-updates");
        game.moves_on_to_the_stage(THE_NEXT_STAGE);
        game.frames_until("the stage's build", A_BUILD, || {
            game.image().the_loader_is_running()
        });
        game.frame();
        assert_eq!(
            game.image().what_the_player_is_doing(),
            COMING_IN,
            "the stage's build did not leave the player coming in",
        );
        game.frames_until("the player's first update", A_BUILD, || {
            game.image().what_the_player_is_doing() != COMING_IN
        });
        assert_eq!(
            the_player(&game),
            (UNTOUCHABLE, UNTOUCHABLE_FOR - 1),
            "the player's first update did not put it in the state nothing reaches it in",
        );
        let (frames, then) = frames_in(&game, UNTOUCHABLE, UNTOUCHABLE_FOR as u32 * 2);
        assert_eq!(
            (frames, then),
            (UNTOUCHABLE_IS_SEEN_FOR - 1, PLAYING),
            "the player did not play again on its 240th update out of the build",
        );
    });
}

/// **A hit, in the three states the game takes it through**: thirty updates of the hit, the life taken and the
/// player back at the start coming in for thirty, and then untouchable — and not moving through the first two,
/// with a direction held all the way.
#[test]
fn a_hit_of_th08_is_the_hit_then_the_player_coming_in_then_untouchable() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-players-hit");
        let lives = game.state().lives;
        let deaths = game.state().deaths;
        let hit_at = (100.0, 300.0);
        game.image().the_player_is_at(hit_at);
        game.image().the_player_has_been_hit();
        game.keyboard().set(keys::UP, true);

        // The first update of the hit, which moves the player nowhere on the held direction.
        game.frame();
        assert_eq!(
            (
                game.image().what_the_player_is_doing(),
                game.image().where_the_player_is()
            ),
            (HIT, hit_at),
            "the player moved while it was hit",
        );
        let (frames, then) = frames_in(&game, HIT, A_HIT_IS_SEEN_FOR * 2);
        assert_eq!(
            (frames + 1, then),
            (A_HIT_IS_SEEN_FOR, COMING_IN),
            "the hit did not stand until its count reached thirty",
        );
        let state = game.state();
        assert_eq!(
            (state.lives, state.deaths),
            (lives - 1, deaths + 1),
            "the hit run out did not take the life",
        );
        let start = (192.0, 384.0);
        assert_eq!(
            game.image().where_the_player_is(),
            start,
            "the player did not come in at the stage's start",
        );

        // And coming in, which moves it nowhere either.
        game.frame();
        assert_eq!(
            (
                game.image().what_the_player_is_doing(),
                game.image().where_the_player_is()
            ),
            (COMING_IN, start),
            "the player moved while it was coming in",
        );
        let (frames, then) = frames_in(&game, COMING_IN, COMING_IN_IS_SEEN_FOR * 2);
        assert_eq!(
            (frames + 2, then),
            (COMING_IN_IS_SEEN_FOR, UNTOUCHABLE),
            "the player did not come in for thirty updates before it was untouchable",
        );
        // **And moves again on the update it is untouchable**, the state its input is read in again.
        assert!(
            game.image().where_the_player_is().1 < start.1,
            "the player did not move on the held direction once it was untouchable",
        );
        game.keyboard().set(keys::UP, false);
    });
}
