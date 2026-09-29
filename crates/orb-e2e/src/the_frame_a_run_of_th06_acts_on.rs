//! **The keys a frame of th06 is played with reach that frame's own update**, with nothing of orb's doing it.
//!
//! What `the_frame_a_run_of_th07_acts_on.rs` asks of that game, asked of this one — and the answer is the
//! other way round. th07's frame of input lag is a copy between two words made by a job that runs after the
//! player's, so orb hands the read over itself. th06 has one word: `Supervisor::OnUpdate`, at chain priority
//! 0, moves `g_CurFrameInput` (0x69d904) into `g_LastFrameInput` (0x69d908) and reads the keyboard into the
//! first, and `Player::OnUpdate` at 7 reads that same word. So `Game::hand_the_read_to_the_run` does nothing
//! for this game, and what says that is right is the player moving on the frame the key was held on.
//!
//! **And the pair is what an edge is read off**, which is what a bomb and every `WAS_PRESSED` of the game
//! is: the frame before's word beside this frame's, so a key held over two frames is a press on the first and
//! a hold on the second. A hand-over that wrote the word and left the pair would read every frame as a hold.

use crate::fake::th06::{Fake, SPEED, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// The bit th06's own read leaves for up — `BUTTON_UP`, 0x0010 — which is what the player's own job tests.
const UP: u16 = 0x10;

/// Where the player is, which is what a direction in the word moves.
fn player(game: &Fake) -> (f32, f32) {
    game.image().reproducing_now().player
}

#[test]
fn a_frame_of_th06_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-frame-it-acts-on", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.in_a_pointdevice_run();

        // Up held, and the one frame it is held for run: the word the game's read left holds it, and the
        // player has moved up by the step a held direction takes — on this frame, not the next.
        let (x, y) = player(&game);
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().input_now() & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );
        assert_eq!(
            player(&game),
            (x, y - SPEED),
            "the player did not move on the frame the key was held on, which is a frame of input lag",
        );
        assert_eq!(
            game.image().last_input_now() & UP,
            0,
            "the frame before's word holds a key that was not down on it",
        );

        // **And the pair an edge is read off is still a pair**: held over a second frame, the word beside the
        // one the run acts on holds it too, which is a hold and not a second press.
        game.frame();
        assert_eq!(
            (
                game.image().input_now() & UP,
                game.image().last_input_now() & UP
            ),
            (UP, UP),
            "the word beside the one the run acts on is not the frame before's",
        );
        assert_eq!(
            player(&game),
            (x, y - 2.0 * SPEED),
            "the player did not move on the second frame the key was held on",
        );

        // And the key let go of reaches the run on its own frame too: the player stops where it was, and the
        // word beside holds the key that was down on the frame before.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.image().input_now() & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
        assert_eq!(
            game.image().last_input_now() & UP,
            UP,
            "the frame before's word does not hold the key that was down on it",
        );
        assert_eq!(
            player(&game),
            (x, y - 2.0 * SPEED),
            "the player moved on a frame no key was held on",
        );
    });
}
