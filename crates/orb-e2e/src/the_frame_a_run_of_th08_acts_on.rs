//! **The keys a frame of th08 is played with reach that frame's own update, and the word beside them is the
//! frame before's.**
//!
//! th08's gap is th07's, one priority further along, and it is written in the chain priorities its jobs run
//! at:
//!
//! | priority | what |
//! | --- | --- |
//! | 0 | the supervisor's own job, which reads the keys — `Th08`'s `INPUT_READ` — into `[0x164d528]` (0x4455f5) |
//! | 9 | the player's own job, which moves on **the word the run acts on**, `[0x164d52c]` (0x44aee8), and bombs where its bit `0x2` differs from the one it acted on, `[0x164d534]` (0x44c8e1 through 0x44c903) |
//! | 0x11 | the run's replay, whose element copies the one into the other (0x452332) and the read into the first (0x452340) |
//!
//! So left to the game the player moves on the word the element left a frame ago, which a whole run of the
//! real game measured: the move from one frame to the next followed the word orb chose on the earlier of the
//! two 5,104 times and contradicted it 16. What orb does about it is th07's answer — hand the read to the run
//! where the read happens, with the word acted on moved along beside it, so that what the player reads at 9 is
//! this frame's word and the pair it bombs off is this frame's against the frame before's.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit this game's own read leaves for up, which the player's job tests first, with left, at 0x44aeef.
/// Written out here rather than taken from `Th08`'s own, which keeps the same number: a test that read its
/// bits off the code under test would agree with it whatever either said.
const UP: u16 = 0x10;

/// And the bomb's, which is the bit the player's job holds the two words against each other for.
const BOMB: u16 = 0x2;

/// A run placed in its first stage, with nothing fired at the player and the frames a placed run comes up
/// over let go by.
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

#[test]
fn a_frame_of_th08_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-frame-it-acts-on");
        let was = game.image().where_the_player_is();

        // Up held, and the one frame it is held for run: what the game's own read leaves is the word with that
        // bit in it, which is the half of this nothing was ever wrong with.
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().the_word_the_keys_read() & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );

        // **And the player has moved on it on that same frame**, which is the half a frame of lag breaks: the
        // player's job at 9 reads the word the replay's element at 0x11 has not copied the read into yet.
        assert_eq!(
            game.what_the_player_read().0 & UP,
            UP,
            "the run acts on a word the key is not in yet, which is a frame of input lag",
        );
        let now = game.image().where_the_player_is();
        assert!(
            now.1 < was.1,
            "the player is still where they were on the frame up was held: {was:?} against {now:?}",
        );

        // And the key let go of reaches the run on its own frame too, which is the same thing said the other way
        // round: a word that lags going up is a player that keeps moving after the key is released.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.what_the_player_read().0 & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
    });
}

/// **And the pair a bomb is taken from is still a pair**: the word the run acted on holds the frame before's
/// as the player's job reads it, so a key held over two frames is a press on the first and a hold on the
/// second. Moved on without it, every frame would read as a hold and no bomb would ever go off.
#[test]
fn the_pair_a_bomb_of_th08_is_taken_from_is_this_frames_word_and_the_frame_befores() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-pair-a-bomb-is-taken-from");

        game.keyboard().set(keys::X, true);
        game.frame();
        assert_eq!(
            (
                game.what_the_player_read().0 & BOMB,
                game.what_the_player_read().1 & BOMB
            ),
            (BOMB, 0),
            "the frame the bomb key went down on is not a press to the player's own job",
        );

        game.frame();
        assert_eq!(
            (
                game.what_the_player_read().0 & BOMB,
                game.what_the_player_read().1 & BOMB
            ),
            (BOMB, BOMB),
            "the second frame the bomb key is held on is not a hold to the player's own job",
        );

        game.keyboard().set(keys::X, false);
        game.frame();
        assert_eq!(
            (
                game.what_the_player_read().0 & BOMB,
                game.what_the_player_read().1 & BOMB
            ),
            (0, BOMB),
            "the frame the bomb key came up on does not hold the frame before's word beside it",
        );
    });
}
