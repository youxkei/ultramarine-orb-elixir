//! **A region of a chapter of th10 that cannot be made writable: named, and the rest of the restore still
//! happens.**
//!
//! The same claim `a_region_a_restore_of_th08_cannot_write.rs` makes of that game, over the regions *this*
//! game is in: its `.data` and the heap blocks beside it, so one region refusing is a share of a run to lose and
//! a restore that gave up at the first refusal would leave the game with half of one chapter and half of
//! another.
//!
//! **Driven by a recipe**, as every e2e test over a run of this game is: the recipe's own rules do the dying
//! and the answering, which is what a launch of the real one does.

use crate::fake::th10::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

/// How long a stage of the laid-out game lasts here: past its midboss's arrival at frame 1200, which is the
/// stage's second chapter.
const A_STAGE: u32 = 4_500;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the recipe never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  ")
    );
}

#[test]
fn a_region_of_th10_that_cannot_be_committed_is_named_and_the_chapter_still_comes_back() {
    in_its_own_process(|| {
        // A chapter reached, then a hit taken and the chapter retried — the laid-out stage's first bullet, at
        // 480, falling inside stage 1's first chapter and inside the frames the hit is taken over.
        let recipe = format!(
            "{TO_THE_SHOT}{TO_A_RUN}\
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 900 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th10-a-region-refused", recipe);
        let log = game.log();

        // The chapter taken, which is what there is to put back.
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });

        // One of the regions a snapshot of this game covers, declared un-committable — the last of them rather
        // than the first, the first being the game's own data and a run that lost that would be no run at all.
        let regions = game.sim().space().game_regions(&game.image().data());
        assert!(
            regions.len() > 1,
            "this game is one region, so the one refused would be its data",
        );
        let (base, len, _) = *regions
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
