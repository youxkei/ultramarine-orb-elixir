//! **The keys a frame of th06nc is played with reach that frame's own update, and the word beside them is
//! the frame before's.**
//!
//! th07's own answer is `the_frame_a_run_of_th07_acts_on.rs`, and there it is a fix: that game reads the
//! keys at chain priority 0, moves the player at 8 and only copies the read into the word the run acts on
//! at 0x10, so the player moves on the frame before's keys until orb hands the read to the run.
//!
//! **This game has no such gap to close either**, and where that was read is `th06nc/asm.md`'s reading of
//! the supervisor: `0x14007b8a6` onwards copies `0x140abae80` into `0x140abae84` and then builds the new
//! word into `0x140abae80`, all at the calc chain's priority nought — and the player's own job is
//! priority 7, behind it in the same walk. So `Th06nc` hands no read to the run and this file says what
//! is left to say: that nothing of orb's puts a frame between the keys and the update, and that the pair
//! an edge is taken from is still a pair.
//!
//! **Which the pair is for**: the menus' own decide is edge-checked against it — `0x140049aa5` masks
//! `0x140abae80` against `0x140abae84` at the main menu and `0x14004d9d7` does the same at the last
//! screen before a stage — so a key held over two frames has to read as a press on the first and a hold
//! on the second. A word moved on without its predecessor would read as a hold on every frame and no
//! decide would ever fire.
//!
//! **And orb's own frame is where the word is written now**, which is this game's own story: its calc
//! walk put the bot's word straight into the game's keys, so `runtime::the_frames_word` — the read a
//! question of orb's is raised on — was never asked, and eight launches of the real game went into
//! finding it. `the_mode_question_over_th06nc` is what holds that; this holds that the write did not cost
//! the frame its own keys.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below is played in, **counted from nought the way `State::stage` is**, which is what
/// `Fake::in_a_run_in_the_stage` is handed: the stage's own build counts the manager's field on, so what
/// goes in before it is the nought-counted number.
const THE_FIRST_STAGE: i32 = 0;

/// The bit this game's own read leaves for up. Written out here rather than taken from `Th06nc`'s own
/// `button` module, which keeps the same number: a test that read its bits off the code under test would
/// agree with it whatever either said.
const UP: u16 = 0x0010;

/// The state a run is played in, and how many frames of a stage go by before the player is in it: the
/// one update of state 1, which a stage begins in, and the invulnerability behind it.
const PLAYING: u8 = 0;
const A_STAGES_FIRST_FRAMES: u32 = 1 + 0xf0;

/// And a second of frames on top of that as the wall this test puts up, rather than the count itself:
/// what is under test here is the word a frame is played with and not which frame the invulnerability
/// ends on.
const A_SECOND: u32 = 60;

#[test]
fn a_frame_of_th06nc_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-the-frame-it-acts-on",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        // **With nothing fired at the player**, whose hit would put them in the state a respawn is and
        // move them on nobody's word: what a stage of this laid-out game fires is
        // `Fake::fires_at_the_player`'s, and a run being read for its keys is one nothing is shooting at.
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        // **Past the frames a stage begins with**, in which the player is walking back in and moves on no
        // word at all.
        //
        // **Counted out rather than waited on a state**, which is what the first shape of this did: the
        // state a run is played in is also the state a stage *begins* in — the respawn holds the player
        // without leaving that state — so a wait on it came back on the stage's first frame and the word
        // held then moved nobody.
        for _ in 0..A_STAGES_FIRST_FRAMES + A_SECOND {
            game.frame();
        }
        assert_eq!(
            game.image().the_players_state(),
            PLAYING,
            "the player is not in the state a run is played in",
        );
        let was = game.image().where_the_player_is();

        // Up held, and the one frame it is held for run.
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().the_word_the_run_acts_on() as u16 & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );
        // **And the player has moved on it on that same frame**, which is the half a frame of lag would
        // break: the word is read at priority 0 and the player's own job is behind it in the same walk.
        let now = game.image().where_the_player_is();
        assert!(
            now.1 < was.1,
            "the player is still where they were, so the run acted on a word the key is not in yet: \
             {was:?} against {now:?}",
        );

        // **And the pair an edge is taken from is still a pair**: the word beside the one the run acts on
        // holds the frame before's, so the second frame of a held key is a hold and not another press.
        game.frame();
        assert_eq!(
            (
                game.image().the_word_the_run_acts_on() as u16 & UP,
                game.image().the_word_before() as u16 & UP,
            ),
            (UP, UP),
            "the word beside the one the run acts on is not the frame before's",
        );

        // And the key let go of leaves the word on its own frame, which is the same thing said the other
        // way round: a word that lags going up is a shot that keeps firing after the key is released.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.image().the_word_the_run_acts_on() as u16 & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
        assert_eq!(
            game.image().the_word_before() as u16 & UP,
            UP,
            "the frame before's word does not hold the key that was down on it",
        );
    });
}
