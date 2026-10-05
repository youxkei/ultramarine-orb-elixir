//! **The keys a frame of th07 is played with reach that frame's own update.**
//!
//! th07 is known for a frame of input lag beyond what its loop costs, and where it is is written in the
//! chain priorities its own jobs are registered at:
//!
//! | priority | what |
//! | --- | --- |
//! | 0 (0x43905a) | the supervisor's own job, which reads the keys — `Th07`'s `INPUT_READ` — and leaves the word at `G_INPUT_READ` |
//! | 8 (0x442ab6) | the player's own job, which reads the word **the run acts on** and moves on it (0x43ee8c, testing the direction bits) |
//! | 0x10 (0x443b9a) | the recorder, whose first act is to copy the read's word into the word the run acts on (0x442d07) |
//!
//! So the player moves on the word the copy left a frame ago: the keys of a frame reach the run one frame
//! late, whatever the loop around them does. **orb's own frame loop is not this** — that one is the frame
//! between the update and the present, and it is gone already — so a launch with orb still had this one.
//!
//! What orb does about it is hand the read to the run where the read happens, which is priority 0: the word
//! the player reads at priority 8 is then the word the keyboard gave this frame, and the recorder's own copy
//! at 0x10 writes the same value over it. The two the game keeps beside them — last frame's of each — are
//! kept the way the recorder keeps them, since what a run reads an *edge* off is the pair.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit th07's own read leaves for up, which is what the player's own job tests at 0x43ee93.
const UP: u16 = 0x10;

#[test]
fn a_frame_of_th07_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-frame-it-acts-on",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // Up held, and the one frame it is held for run: what the game's own read leaves is the word with that
        // bit in it, which is the half of this nothing was ever wrong with.
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().the_word_the_read_left() & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );

        // **And the word the run acts on holds it on the same frame**, which is what the player's own job reads
        // at priority 8: left to the recorder's copy at 0x10, that word still holds the frame before's and the
        // player moves a frame late.
        assert_eq!(
            game.the_player_read() & UP,
            UP,
            "the run acts on a word the key is not in yet, which is a frame of input lag",
        );

        // **And the pair an edge is read off is still a pair**, which is what a bomb is: the word beside it holds
        // the frame before's, so a key held over two frames is a press on the first and a hold on the second.
        // Moved on without it, every frame would read as a hold and no bomb would ever go off.
        game.frame();
        assert_eq!(
            (
                game.the_player_read() & UP,
                game.word_the_run_acted_on() & UP,
            ),
            (UP, UP),
            "the word beside the one the run acts on is not the frame before's",
        );

        // And the key let go of reaches the run on its own frame too, which is the same thing said the other way
        // round: a word that lags going down is a shot that keeps firing after the key is up.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.the_player_read() & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
        assert_eq!(
            game.word_the_run_acted_on() & UP,
            UP,
            "the frame before's word does not hold the key that was down on it",
        );
    });
}
