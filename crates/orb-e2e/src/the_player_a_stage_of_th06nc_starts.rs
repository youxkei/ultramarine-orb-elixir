//! **What a stage of th06nc leaves its player as, which is 270 frames nothing can kill them in.**
//!
//! th06 1.02h's own answer is `the_player_a_stage_starts.rs` and th06c's is
//! `the_player_a_stage_of_th06c_starts.rs`; this game has the same one, read off its own exe rather than
//! carried over — `th06nc/asm.md`'s *The 270 frames of a stage nothing can reach its player in*:
//!
//! - **`0x14006841c` is the register.** Where `[0x1404f27b5]` is nought it leaves the state and the count
//!   at nought, and otherwise `0x14006842b` puts **1** in `cl` and **0x78** in `eax` and `0x140068432`
//!   and `0x14006843f` write the pair, with `-999` into `+0x7854` beside it.
//! - **The walk-in lasts thirty frames.** `0x140068da7`'s `cmp ecx,0x1e` is the gate, and `0x140068f8b`'s
//!   `cmp r8d,0x1e` is the one past which `0x140068f95` writes the state **3** and `0x140068fb1` the count
//!   **0xf0**.
//! - **And state 3 counts down to nothing.** `0x140068c6e` tests the state against 3, `0x140068c81` takes
//!   one off the count and `0x140068c91` writes nought over both where it runs out.
//!
//! **So the hit test cannot reach the player for 30 + 240 = 270 frames of every stage**, `0x14006aaa5`
//! gating a hit on that byte being nought. Which is what the second test here is about, and what the
//! first one is is the two numbers the register leaves — the state a chapter cannot be taken in, and a
//! count nobody but state 3 reads.
//!
//! **And the third is the shot the register leaves the player firing, which is none.** The pair at
//! `+0x408` and `+0x40c` is the phase of the shot being fired and the phase the update before had —
//! `th06nc/asm.md`'s row for it — and a stage carries neither in: `0x140068736` writes the idle pair
//! beside the eighty slots `0x140068664`–`0x14006872f` clears. What the test holds is the cycle the press
//! then starts: `0x1400696f4` on the composed word's own `0x1`, `0x140069c6a` a phase an update, and
//! `0x140069c7e` the end of it at thirty.
//!
//! **Read off the exe and not off this tree**, because the laid-out game is both the writer and the
//! reader of the memory orb is driven through: a build that left the player playing would be one every
//! e2e test here agreed with, and every one of them would have a player a bullet can reach on a stage's
//! first frame.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below is played in, **counted from nought the way `State::stage` is**, which is what
/// `Fake::in_a_run_in_the_stage` is handed: the stage's own build counts the manager's field on, so what
/// goes in before it is the nought-counted number.
const THE_FIRST_STAGE: i32 = 0;

/// The player's states, written out here rather than taken from `Th06nc`'s own `player` module, which
/// keeps the same numbers: a test that read them off the code under test would agree with it whatever
/// either said.
const PLAYING: u8 = 0;
const COMING_BACK: u8 = 1;
const UNTOUCHABLE: u8 = 3;

/// What the register leaves on the count, what the walk-in leaves there, and how long the walk-in is.
const THE_REGISTER_LEAVES: i32 = 0x78;
const THE_WALK_IN_LEAVES: i32 = 0xf0;
const THE_WALK_IN_TAKES: u32 = 0x1e;

/// **The pair a player firing nothing holds, and how many updates a cycle lasts** — `0x140069c94`'s
/// qword and `0x140069c7e`'s thirty. Written out here for this file's own reason: a number read off the
/// code under test is one the test agrees with whatever either says.
const NOT_FIRING: (i32, i32) = (-1, -999);
const A_FIRING_CYCLE_LASTS: i32 = 30;

/// How many frames of a run the cycle below is read over — more than two of them, so that the sequence
/// is held across an end and not only inside one.
const FRAMES_THE_CYCLE_IS_READ_OVER: u32 = 90;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// A stage's own build leaves the player coming back, with the register's count under it — and orb reads
/// that as a run in the middle of something.
#[test]
fn a_stage_of_th06nc_starts_its_player_coming_back() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-players-stage-start");
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // Read before a frame is run, which is what "what the build leaves" means: the register is the
        // stage build's own and the walk-in is the frames after it.
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_player_is_untouchable_for(),
            ),
            (COMING_BACK, THE_REGISTER_LEAVES),
            "the stage's build did not leave the player as its own register leaves them",
        );
        assert!(
            game.state().unsettled,
            "a player coming back reads as a run standing somewhere a chapter can be taken",
        );
        assert_eq!(
            game.image().the_shot_phase(),
            NOT_FIRING,
            "the stage's build did not leave the player firing nothing",
        );
    });
}

/// **And the shot the player fires runs in cycles of thirty updates**, started by the press and not
/// carried in from the stage before: the phase steps one an update to the cycle's end and is idle for
/// the update after it, which is the update a press starts the next cycle in.
#[test]
fn the_shot_of_a_stage_of_th06nc_fires_in_cycles_of_thirty_updates() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-shots-own-cycle");
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // Past the walk-in and the invulnerability, the press being the half a player in either skips.
        // **One frame more than the 270 those take**, the stage's own build spending a frame of its own
        // that its clock does not count — where the test above waits from the walk-in's end instead.
        game.frames_until(
            "the player playing",
            THE_WALK_IN_LEAVES as u32 + THE_WALK_IN_TAKES + 1,
            || game.image().the_players_state() == PLAYING,
        );

        let mut phases = Vec::new();
        for _ in 0..FRAMES_THE_CYCLE_IS_READ_OVER {
            game.frame();
            phases.push(game.image().the_shot_phase().0);
        }

        // **Nought is a phase no frame ends on**, the advance running in the same update as the press: a
        // cycle's own updates are read as 1 through 29 and the idle one as the pair above.
        for (frame, phase) in phases.iter().enumerate() {
            assert!(
                *phase == NOT_FIRING.0 || (1..A_FIRING_CYCLE_LASTS).contains(phase),
                "frame {frame} of a run left the shot on phase {phase}, which is no phase of a cycle \
                 of {A_FIRING_CYCLE_LASTS}",
            );
        }
        for (frame, pair) in phases.windows(2).enumerate() {
            let [before, now] = [pair[0], pair[1]];
            let wanted: &[i32] = match before {
                // The cycle's last update hands over to the idle one.
                phase if phase + 1 == A_FIRING_CYCLE_LASTS => &[NOT_FIRING.0],
                // And an idle update is followed by the first of the next cycle, or by another idle one
                // where the rules let the shot go — which they do one frame in sixty, see `orb_bot`'s
                // own `SHOT_LET_GO_EVERY`.
                phase if phase < 0 => &[1, NOT_FIRING.0],
                phase => &[phase + 1],
            };
            assert!(
                wanted.contains(&now),
                "the shot went from phase {before} to {now} between frames {frame} and {}, where a \
                 cycle of {A_FIRING_CYCLE_LASTS} allows {wanted:?}",
                frame + 1,
            );
        }
        assert!(
            phases.contains(&NOT_FIRING.0),
            "no cycle ended in {FRAMES_THE_CYCLE_IS_READ_OVER} frames of a run, so what the sequence \
             above holds is one cycle's middle and not its end",
        );
    });
}

/// And nothing reaches the player until the walk-in and the invulnerability after it have both run out,
/// which is 270 frames of every stage.
#[test]
fn nothing_of_a_stage_of_th06nc_reaches_its_player_for_the_first_270_frames() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-players-first-frames");
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // The walk-in, which is the state the build left and the frames it lasts.
        for frame in 0..THE_WALK_IN_TAKES {
            game.frame();
            let state = game.image().the_players_state();
            assert!(
                state == COMING_BACK || (state == UNTOUCHABLE && frame + 1 == THE_WALK_IN_TAKES),
                "the player is in state {state} on frame {} of a walk-in of {THE_WALK_IN_TAKES}",
                frame + 1,
            );
        }
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_player_is_untouchable_for(),
            ),
            (UNTOUCHABLE, THE_WALK_IN_LEAVES),
            "the walk-in did not hand the player over to the count the game's own gives it",
        );

        // And the invulnerability after it, run out frame by frame: the player is playing on the frame
        // the count reaches nothing and in no frame before it.
        game.frames_until(
            "the player playing",
            THE_WALK_IN_LEAVES as u32 + THE_WALK_IN_TAKES,
            || game.image().the_players_state() == PLAYING,
        );
        assert_eq!(
            game.state().stage_frames,
            THE_WALK_IN_TAKES + THE_WALK_IN_LEAVES as u32,
            "the player began playing on another frame of the stage than the count says",
        );

        // **And the stage's own fire passed through them**, which is what those frames are for: a bullet
        // aimed at a player who has not moved reaches them inside this window, and the hit test is gated
        // on a state none of it was in.
        assert_eq!(
            game.state().deaths,
            0,
            "something reached the player inside the frames nothing can",
        );
    });
}
