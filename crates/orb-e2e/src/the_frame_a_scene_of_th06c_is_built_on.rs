//! **The frame a scene of th06c is built on is a frame that scene is updated on — and the input words are
//! left holding whatever was down, which th06 1.02h zeroes.**
//!
//! Two readings off the exe, both in `th06c/asm.md`:
//!
//! - **The calc walk reads an element's `+0x28` after its callback has returned**, `0x140025559` against the
//!   call at `0x140025538`, and the chain is a list in priority order. So a job registered by the job being
//!   run — which is what a scene building itself is — is reached in that same walk, and a scene gets its own
//!   first update on the frame it was built on. Which is why `State::stage_frames` reads **one** and not
//!   nought on the first frame orb sees a stage.
//! - **And nothing of a transition zeroes the word a run acts on.** Three places in the whole exe write
//!   `0x14090a110` or `0x14090a114`: the front end's own state nought, where `th06TL.dat` is loaded; the
//!   input job's copy and read; and the replay playback. So where th06 1.02h clears the pair at the end of
//!   every state of its supervisor — `the_frame_a_scene_is_built_on.rs`, whose third test is that clear —
//!   this game carries the keys straight across a transition, and the new scene's first update acts on
//!   whatever is held.
//!
//! **Which is a difference orb has to live with rather than one to fix**: a decide still down on the frame a
//! stage is built is a decide this game's own menus would spend again, and what keeps it from being spent is
//! the edge each of them reads off the pair. The second test is that behaviour written down, so that a
//! reading of this game which assumed th06's fails here.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below starts in, orb's own 0-based number for it, and the one it walks into.
const THE_FIRST_STAGE: i32 = 0;
const THE_NEXT_STAGE: i32 = 1;

/// How long each stage lasts here, in the game's own frames — short enough that a test walks into the stage
/// after the first, long enough that the walk is a stage and not a build.
const STAGE_FRAMES: u32 = 200;

/// The frames the walk into the next stage is given past the first stage's last.
const A_TRANSITION: u32 = 60;

/// And how long the logo of this laid-out game stands before the front end is asked for, which the wait for
/// the front end's own build is given on top of.
const THE_LOGO: u32 = 60;

/// The bit this game's own read leaves for down, which the walk at `0x14004ac69` tests second of the four.
const DOWN: u16 = 0x0020;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// The stage a transition builds is a stage being played on that very frame: its own clock already at its
/// first frame, and the scene the one a run is played in.
#[test]
fn a_stage_of_th06c_is_played_on_the_frame_it_was_built_on() {
    in_its_own_process(|| {
        let game = launched("th06c-the-frame-a-stage-is-built-on");
        game.stages_last(STAGE_FRAMES);
        // **Nothing fired at the player**, which is what lets a stage be played out here at all: the stage's
        // own fire reaches a player who stands still at frame 185, and a death in this mode is a retry
        // offered — a question nobody in this test answers, and the game stands on that frame while it is up.
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        game.frames_until(
            "the first stage played out",
            STAGE_FRAMES + A_TRANSITION,
            || game.state().stage == THE_NEXT_STAGE,
        );

        // Read on the frame the condition above came true on, which is what "on the frame it was built"
        // means: `frames_until` stops there, and one more frame of the run is another frame's state.
        let built = game.state();
        assert_eq!(
            built.stage_frames, 1,
            "the stage a transition built did not get its own first update on the frame it was built on: \
             {built}",
        );
        assert!(
            built.playing && built.in_run && built.in_game,
            "the frame the stage was built on is not one a run is being played on: {built}",
        );

        // And the frame after it is the stage's second, which is what says the first was not spent on the
        // build.
        game.one_frame();
        assert_eq!(
            game.state().stage_frames,
            2,
            "the stage's own clock did not move on the frame after the build",
        );
    });
}

/// And the front end's own first update runs on the frame it was built on, which is the same claim at the
/// transition every launch begins with.
#[test]
fn the_front_end_of_th06c_is_updated_on_the_frame_it_was_built_on() {
    in_its_own_process(|| {
        let game = launched("th06c-the-frame-the-front-end-is-built-on");
        // The launch begins on its logo's own scene with the front end asked for and not built. The frame the
        // supervisor builds it on is the frame it acts on that ask — `the_scene_last_acted_on` catching up
        // with `the_scene_now` is that, and it is the supervisor's own last act of the frame.
        game.frames_until("the front end built", THE_LOGO + A_TRANSITION, || {
            let scene = game.image().scene_of_the_front_end();
            game.image().the_scene_last_acted_on() == scene && game.image().the_scene_now() == scene
        });
        assert_eq!(
            game.image().frames_the_screen_has_been_up(),
            1,
            "the front end did not get its own first update on the frame it was built on",
        );
    });
}

/// And the word a run acts on carries a held key across a transition, the pair beside it with it.
#[test]
fn a_scene_of_th06c_is_built_with_the_words_a_key_is_still_in() {
    in_its_own_process(|| {
        let game = launched("th06c-the-frame-the-words-stand");
        game.stages_last(STAGE_FRAMES);
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // Held from before the transition and never let go, so that nothing about what the keyboard is doing
        // changes on the frame the scene is built: what is under test is what the *game* does to the words.
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        assert_eq!(
            game.image().the_word_the_run_acts_on() as u16 & DOWN,
            DOWN,
            "the game's own read did not see the key being held",
        );

        game.frames_until(
            "the first stage played out",
            STAGE_FRAMES + A_TRANSITION,
            || game.state().stage == THE_NEXT_STAGE,
        );
        assert_eq!(
            (
                game.image().the_word_the_run_acts_on() as u16 & DOWN,
                game.image().the_word_before() as u16 & DOWN,
            ),
            (DOWN, DOWN),
            "the frame the stage was built on came out with the words cleared, which this game clears \
             only where its title's data is loaded",
        );
        game.keyboard().set(keys::DOWN, false);
    });
}
