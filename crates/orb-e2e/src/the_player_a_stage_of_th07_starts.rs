//! **What a stage of th07 leaves its player as, which is nothing that keeps them alive.**
//!
//! th06's own answer is `the_player_a_stage_starts.rs`: spawning, 120 on the invulnerability count, and a first
//! update that turns it into 240 updates nothing can kill the player in. **th07 has no such thing**, and that is
//! why this file is not a second copy of that one — it says the opposite, and where it was read is here:
//!
//! - **0x4429d0 is the player's own register**, and its two callers are a run's build (0x42e9d6) and *every*
//!   stage's (0x42ef03, inside the build that adds one to the stage word at 0x42ef50). It fills 0x2df9e dwords
//!   from 0x4bdad8 with zeroes — so the state byte at `+0x2408` is 0, the state a run is played in — and then
//!   writes the timer at `+0x16a00` outright: `[+8]` and `[+4]` to nothing, `[+0]` to -999. Its three chain
//!   callbacks (0x441fb0, 0x4420b0, 0x442350) follow, and none of them writes either field.
//! - **The hit test's only guard is that state byte.** 0x43e353 lets a player in state 4 break the border
//!   (0x441bd0), 0x43e375 returns without a death for any other non-zero state, and 0 falls through to the death
//!   at 0x43edc0 — **no count of frames is read anywhere in it**. So what makes th07's player unkillable is
//!   being in one of the four states that are not 0, and nothing else.
//! - **What puts the player in state 3 is a bomb, a death's end or a border's**, never a stage's start: the
//!   twelve routines at 0x408710 through 0x40da80 write it beside sound 0xd, the death handler at 0x4412a6 once
//!   its 30 updates are out, and the border's end at 0x441785 and its breaking at 0x441d5c. The frames under it
//!   are ticked by 0x4394c7 and the state goes back to 0 at 0x4413fa where they run out.
//!
//! **So a stage of this game begins with a player anything can kill**, and there is no e2e test to be written
//! about how long that is not so for. What is left to say is what a build leaves in the two fields orb writes for
//! `--clear` — see `a_clear_on_demand_of_th07.rs` — and that is what this file asserts: the state a run is played
//! in, and a timer holding nothing however the stage before it ended.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage this run moves on to, orb's own 0-based number for it: the one after the stage a run starts on, so
/// that the build below is a build with a stage behind it.
const THE_NEXT_STAGE: i32 = 1;

/// The state a run is played in, and the one a death's end leaves a player untouchable in. Written out here
/// rather than read off `Th07`, which keeps both: a test that took its numbers from the code under test would
/// agree with it whatever either said.
const PLAYING: i8 = 0;
const COMING_BACK: i8 = 3;

/// And the frames a death's end leaves under that state, which is what the stage before this one is left
/// holding: 0x4411c0's write beside the state.
const FRAMES_A_RESPAWN_LEAVES: i32 = 0xf0;

#[test]
fn a_stage_of_th07_starts_its_player_playing_with_nothing_left_on_the_timer() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-players-stage-start",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The player as a stage that was played through leaves them: walked back in after a death, with frames
        // still to run under the state that walk is made in.
        game.image()
            .the_player_is_coming_back(FRAMES_A_RESPAWN_LEAVES);
        game.one_frame();
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_state_has_left(),
            ),
            (COMING_BACK, FRAMES_A_RESPAWN_LEAVES),
            "the player is not where a respawn left them, so the stage below starts from nothing",
        );
        // And what orb reads of that: a player playing, the death being over. 0x4411c0 holds the dying state for
        // its 30 updates and then writes this one with 0xf0 under it (0x4412a6), and the update moves the player
        // in it (0x43ee50 at 0x44203b) as in the state a run is played in — the frames are untouchable, and not
        // a death still going.
        assert!(
            !game.state().unsettled,
            "the frames after a death read as the death still going",
        );

        // The next stage built over it, which is where the player's own register runs again.
        game.builds_the_stage(THE_NEXT_STAGE);
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_state_has_left(),
            ),
            (PLAYING, 0),
            "the stage's build left the player as the stage before it ended, not as its own register leaves them",
        );
        // Which is a run standing somewhere a chapter can be taken, the state being the whole of what says so.
        assert!(
            !game.state().unsettled,
            "the stage orb was handed reads as a run in the middle of something",
        );
        game.frames_until("the stage's own first chapter", 600, || {
            log.said(&format!(
                "stage {} chapter 1 (stage start)",
                THE_NEXT_STAGE + 1
            ))
        });
    });
}
