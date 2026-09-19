//! **The frame a scene of th06nc is built on: the scene's own job runs in it, and the word a run acts on
//! is nought.**
//!
//! Two things about that one frame, both read off the exe and both things orb rests on.
//!
//! **The scene's own first update is in the same frame as its build.** `0x14003a8e3` calls the stage's
//! build from inside `0x14003a810`, which is the chain's own build, and the job it registers is walked by
//! the same walk — so the stage's clock reads one on the frame the build ran and not nought. Which is
//! what a chapter's own frame is: `Chapters` writes a chapter down against `Game::stage_frame`, and a
//! build whose job waited a frame would put chapter 1 at frame nought and the restore one frame short of
//! where it was taken.
//!
//! **And the word a run acts on is zeroed by the transition, together with the frame before's.** All six
//! writers of `0x140a6ec60` are read — see `th06nc/asm.md` — and two of them are cases of the front
//! end's own jump table at `0x140047ad2`, each writing nought into that word, into `0x140a6ec64` and into
//! the two repeat statics beside them. So the pair an edge is taken from starts a run empty, and a key
//! held from before the run is not a press inside it.
//!
//! **Which is the other half of what `Game::swallow_input` rests on.** That fills the word with every bit
//! so that the frame the game carries on into can find no edge — see `Th06nc::swallow_input`, and
//! `the_frame_a_run_of_th06nc_acts_on` for the pair inside a run. A transition is where that fill is
//! undone, and it is undone by the game and not by orb: what this file says is that the laid-out game
//! undoes it the same way, so that an e2e test about a press held across one is asking the question the
//! real game answers.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the laid-out game is up before its logo comes, with room for the front end to be built.
const UNTIL_THE_FRONT_END: u32 = 90;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// **The scene's own job runs on the frame its build did**, which is what makes a chapter's frame the
/// frame it was written down at.
#[test]
fn a_scene_of_th06nc_is_updated_on_the_frame_it_was_built_on() {
    in_its_own_process(|| {
        let game = launched("th06nc-scene-built-on");
        game.frames(UNTIL_THE_FRONT_END);

        // The build itself, which is orb's hook over `0x14003a9c0` entered with the manager: the stage's
        // clock goes to nought there and no frame of the loop has run yet.
        game.in_a_run_in_the_stage(0);
        assert_eq!(
            game.state().stage_frames,
            0,
            "the build left the stage's clock somewhere other than its own start",
        );

        game.frame();
        assert_eq!(
            game.state().stage_frames,
            1,
            "the scene's own job did not run on the frame it was built on",
        );
    });
}

/// **And the word a run acts on is nought on that frame, with the frame before's** — a key held from
/// before the run reaching neither.
#[test]
fn a_transition_of_th06nc_zeroes_the_word_a_run_acts_on_and_the_frame_before_it() {
    in_its_own_process(|| {
        let game = launched("th06nc-word-at-a-transition");
        game.frames(UNTIL_THE_FRONT_END);

        // A key held down, and long enough for the supervisor's own read to have put it in both words:
        // the read copies this word into the frame before's and then builds the new one, so two frames
        // of a held key is a word and a predecessor that both carry it.
        game.keyboard().set(keys::Z, true);
        game.frames(3);
        assert_ne!(
            game.image().the_word_the_run_acts_on(),
            0,
            "the key held never reached the word a run acts on",
        );
        assert_ne!(
            game.image().the_word_before(),
            0,
            "the key held never reached the frame before's word",
        );

        // And the run asked for, which is the transition: the supervisor is in the front end's scene and
        // wants the run's, which is the pair `fake::th06nc`'s own `builds` is entered with.
        let front_end = game.image().scene_of_the_front_end();
        let run = game.image().scene_of_a_run();
        game.image().is_in_scene(front_end, run);
        game.frame();

        assert_eq!(
            game.image().the_word_the_run_acts_on(),
            0,
            "the transition left the word a run acts on carrying the key that was held",
        );
        assert_eq!(
            game.image().the_word_before(),
            0,
            "the transition left the frame before's word carrying the key that was held",
        );
    });
}
