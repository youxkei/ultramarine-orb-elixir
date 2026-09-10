//! **A playback of th08 that has run out feeds nothing again, and the stage after it is where that is got
//! wrong.**
//!
//! What bounds a playback is the frame it stops feeding on, held beside the buttons, and what it is compared
//! against is the frame of the *stage*. Both are the stage's own numbering, so the two agree for as long as
//! the run stays in the stage it was picked up in: every frame the run reaches after the landing is past the
//! frame the feeding stopped on, and the playback answers nothing.
//!
//! **A stage boundary puts that frame back to nothing while the bound stays where it was** — on this game
//! more plainly than on any other, its stage's frames being counted from each stage's own build — so every
//! frame of the new stage is below it, and `Record::the_playback_is_over`, called where a stage begins, is
//! what stops it. `the_end_of_a_playback_of_th06c` is the same claim over that game, the hook and the record
//! being one for all of them.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-0-ply00a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// **How long a stage of this laid-out game lasts here**: past stage 1's second midstage boundary at script
/// 790, so that the run left in that chapter walks into the second stage without a test sitting through a
/// whole one.
const A_STAGE: u32 = 1_500;

/// How many frames of the stage the run walked into are watched for a frame being fed: past the number the
/// bound left standing would have covered, which is every frame of it below the chapter's own 790.
const WATCHED_FOR: u32 = 900;

/// A launch played out, the game asked to close by its last step.
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

/// What a launch left under orb's directory, re-rooted for the next.
fn left_by(game: &Fake, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            let inside = path.strip_prefix(dir).ok()?.to_path_buf();
            Some((inside, files.get(&path)?))
        })
        .collect()
}

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

/// **A run left in its second chapter**, and the files that launch left behind.
fn a_run_left_behind() -> Vec<(PathBuf, Vec<u8>)> {
    let game = launched(
        "th08-the-end-of-a-playback-left",
        format!(
            "{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        ),
    );
    played_out(&game);
    let files = left_by(&game, game.launch().dir());
    assert!(
        files.iter().any(|(inside, _)| {
            let said = inside.to_string_lossy();
            said.contains("pointdevice_resume") && said.contains(SLOT)
        }),
        "no chapter file of {SLOT} was left: {:?}",
        files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
    );
    files
}

/// **A playback that has landed feeds no frame of the stage after it.**
#[test]
fn a_playback_of_th08_that_has_landed_feeds_no_frame_of_the_stage_after_it() {
    in_its_own_process(|| {
        let files = a_run_left_behind();
        // Picked up — the question this game asks as the run is built, answered with つづきから — and played
        // on into the stage after it, and a stretch of that one.
        let game = launched(
            "th08-the-end-of-a-playback",
            format!(
                "{TO_A_RUN}\
- answer: {{ question: resume, item: 0 }}
  give_up_after: 1800
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 36000
- wait_until: {{ frames: {WATCHED_FOR} }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
            ),
        );
        for (inside, bytes) in &files {
            game.sim().files().put(game.dir().join(inside), bytes);
        }
        let log = game.log();

        // The landing in the stage the run was left in, which is where the bound is set: waited for at the
        // playback's own end, that being the frame the bound is left standing from.
        game.frames_until("the playback run out", FRAMES, || {
            log.said("resume: the playback stopped because")
        });
        assert!(
            log.said("played back with"),
            "the run was picked up and no frame of it was played back:\n  {}",
            log.lines().join("\n  ")
        );

        // And the stage it walks into, where that bound would still cover every frame below it.
        game.frames_until("the stage after the landing", FRAMES, || {
            game.state().stage == 1
        });
        let from = log.written();
        for _ in 0..WATCHED_FOR {
            if game.sim().exited().is_some() {
                break;
            }
            game.frame();
        }
        let fed: Vec<String> = log
            .lines()
            .into_iter()
            .skip(from)
            .filter(|line| line.contains("played back with"))
            .collect();
        assert!(
            fed.is_empty(),
            "the playback that landed in the stage before this one answered for {} frame(s) of it:\n  {}",
            fed.len(),
            fed.join("\n  ")
        );
    });
}
