//! **The keys a frame of th11 is played with reach that frame's own update, and the word beside them is the
//! frame before's.**
//!
//! What `the_frame_a_run_of_th10_acts_on.rs` asks of that game, asked of this one. The priorities its jobs run at
//! put the copy the run acts on ahead of the player:
//!
//! | priority | what |
//! | --- | --- |
//! | 1 | the supervisor's own job, which reads the keys into `[0x4c92a8]` (0x429062) |
//! | `0xb` | the replay's job, which copies the word into the run's own, `& 0xffff`, with the copy before it kept beside it (0x43600d, 0x436008) |
//! | `0x10` | the player's own job, which moves on that copy (0x430293 from 0x43121b) |
//!
//! So the player moves on the word read this same frame, with nothing of orb's in between: what this holds is
//! that the order stands with orb's read of the keys standing in front of the pad's half (0x4579eb).

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit this game's own read leaves for up, which the player's move tests (0x430290). Written out here rather
/// than taken from `Th11`'s own: a test that read its bits off the code under test would agree with it whatever
/// either said.
const UP: u32 = 0x10;

/// And the bomb's, which is the bit the run's copy holds against the copy before it.
const BOMB: u32 = 0x2;

/// A run placed in its first stage and past its build, with nothing fired at the player.
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
    game.frames_until("the stage's first frames", 120, || {
        game.state().in_run && game.state().stage_frames > 0
    });
    game
}

#[test]
fn a_frame_of_th11_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = in_a_run("th11-the-frame-it-acts-on");
        let was = game.image().where_the_player_is();

        // Up held, and the one frame it is held for run: the game's own read leaves the word with that bit in
        // it, and the run's copy has it on the same frame.
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().the_words_the_keys_read().0 & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );
        assert_eq!(
            game.image().the_run_word() & UP,
            UP,
            "the run acts on a word the key is not in yet, which is a frame of input lag",
        );
        let now = game.image().where_the_player_is();
        assert!(
            now.1 < was.1,
            "the player is still where they were on the frame up was held: {was:?} against {now:?}",
        );

        // And the key let go of reaches the run on its own frame too.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.image().the_run_word() & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
    });
}

/// **And the pair a bomb is taken from is still a pair**: the run's copy holds the frame before's beside it
/// (`0x4c93c4`, 0x436008), so a key held over two frames is a press on the first and a hold on the second.
#[test]
fn the_pair_a_bomb_of_th11_is_taken_from_is_this_frames_word_and_the_frame_befores() {
    in_its_own_process(|| {
        let game = in_a_run("th11-the-pair-a-bomb-is-taken-from");
        let pair = || {
            (
                game.image().the_run_word() & BOMB,
                game.image().the_run_word_before() & BOMB,
            )
        };

        game.keyboard().set(keys::X, true);
        game.frame();
        assert_eq!(
            pair(),
            (BOMB, 0),
            "the frame the bomb key went down on is not a press to the run's copy",
        );
        game.frame();
        assert_eq!(
            pair(),
            (BOMB, BOMB),
            "the second frame the bomb key is held on is not a hold to the run's copy",
        );
        game.keyboard().set(keys::X, false);
        game.frame();
        assert_eq!(
            pair(),
            (0, BOMB),
            "the frame the bomb key came up on does not hold the frame before's word beside it",
        );
    });
}

/// **Up and down held together move the player down**, and left and right left: 0x430290 checks a fixed order —
/// the four diagonals, then down (0x20), up, left (0x40) and right — and takes the first that is in the word,
/// where netting the two would stand the player still.
#[test]
fn opposite_directions_held_together_move_th11s_player_the_way_the_game_checks_first() {
    in_its_own_process(|| {
        let game = in_a_run("th11-opposite-directions");
        let (x, y) = game.image().where_the_player_is();
        game.keyboard().set(keys::UP, true);
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        game.keyboard().set(keys::UP, false);
        game.keyboard().set(keys::DOWN, false);
        let (after_x, after_y) = game.image().where_the_player_is();
        assert!(
            after_y > y && after_x == x,
            "up and down together moved the player from {:?} to {:?}, not down",
            (x, y),
            (after_x, after_y),
        );

        game.keyboard().set(keys::LEFT, true);
        game.keyboard().set(keys::RIGHT, true);
        game.frame();
        game.keyboard().set(keys::LEFT, false);
        game.keyboard().set(keys::RIGHT, false);
        let (last_x, last_y) = game.image().where_the_player_is();
        assert!(
            last_x < after_x && last_y == after_y,
            "left and right together moved the player from {:?} to {:?}, not left",
            (after_x, after_y),
            (last_x, last_y),
        );
    });
}
