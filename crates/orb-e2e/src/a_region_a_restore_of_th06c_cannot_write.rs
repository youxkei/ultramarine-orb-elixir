//! **A region of a chapter of th06c that cannot be made writable: named, and the rest of the restore still
//! happens.**
//!
//! The same claim `a_region_a_restore_cannot_write.rs` makes of th06 and
//! `a_region_a_restore_cannot_write_of_th07.rs` of th07, over the regions *this* game is in — and this is the
//! game the claim costs the most on. A chapter of th06c is `7 region(s) of 10142288 bytes`, ten megabytes
//! against th06's few hundred kilobytes, so one region refusing is a larger share of a run to lose and a
//! restore that gave up at the first refusal would leave the game with half of one chapter and half of
//! another.
//!
//! Why the commit happens outside the write loop at all is beside that loop: it runs with the game's other
//! threads suspended, and `VirtualAlloc` is exactly the call that must not happen there.
//!
//! **Driven by a recipe**, as every e2e test over a run of this game is: orb notices the decide for th06c only
//! where a word is being fed, so the retry is answered by the recipe's own rules rather than by a key — see
//! `a_pointdevice_run_of_th06c.rs`.

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

#[test]
fn a_region_of_th06c_that_cannot_be_committed_is_named_and_the_chapter_still_comes_back() {
    in_its_own_process(|| {
        // A chapter reached, then a hit taken and the chapter retried: the recipe's own rules do the dying
        // and the answering, which is what a launch of the real one does.
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th06c-a-region-refused", recipe);
        let log = game.log();

        // The chapter taken, which is what there is to put back.
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });

        // One of the regions a snapshot of this game covers, declared un-committable — the last of them rather
        // than the first, the first being the game's own data and a run that lost that would be no run at all.
        // Which region it is is the game's to say: what the claim is about is that *a* region refusing is
        // survivable, and the list comes out of the same walk the snapshot's does.
        let regions = game.sim().space().game_regions(&game.image().data());
        assert!(
            regions.len() > 1,
            "this game is one region, so the one refused would be its data",
        );
        let (base, len) = *regions
            .last()
            .expect("a game laid out in more than one region");
        game.sim().space().refuses_to_commit(base);

        // The death the rules bring, and the chapter put back after it.
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        // The region is named, with the address and the length, so that whoever reads the log can go and look
        // at what is there.
        assert!(
            log.said(&format!("restore: cannot write {base:#010x}+{len:#x}")),
            "the region that could not be made writable was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And the run carried on: the frames of the stage move after the restore, which is what says the
        // refusal cost the region and not the game.
        //
        // The frames and not every field, because the region refused is one of the game's own and may be the
        // one this run's numbers are in — which is the whole point of the claim. What a refusal costs is
        // whatever was in that region; what it must not cost is the restore.
        let from = game.state().stage_frames;
        for _ in 0..30 {
            game.frame();
        }
        assert!(
            game.state().stage_frames > from,
            "the game stopped being played after a region refused to commit:\n  {}",
            log.lines().join("\n  ")
        );
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.said(fatal),
                "a region that could not be committed took the game down:\n  {}",
                log.lines().join("\n  ")
            );
        }
        played_out(&game);
    });
}
