//! **A playback of th06c that has run out feeds nothing again, and the stage after it is where that was got
//! wrong.**
//!
//! What bounds a playback is the frame it stops feeding on, held beside the buttons, and what it is compared
//! against is the frame of the *stage*. Both are the stage's own numbering, so the two agree for as long as
//! the run stays in the stage it was picked up in: every frame the run reaches after the landing is past the
//! frame the feeding stopped on, and the playback answers nothing.
//!
//! **A stage boundary puts that frame back to nothing while the bound stays where it was.** The stage the run
//! walks into is registered with the buttons of the stage before it thrown away and the frame the feeding
//! stopped on left standing, so every frame of the new stage is below it — and the buttons the player plays
//! there are written down against those same frames as they are played. A frame reached twice is then a frame
//! the playback has an answer for: the chapter retried hands the run the buttons somebody played through it a
//! minute ago, one frame after another, and the run plays itself through the whole stretch it has already
//! been played through. `Record::the_playback_is_over`, called where a stage begins, is what stops it.
//!
//! **The same hook and the same record in all three games**, which is why this file stands beside
//! `the_end_of_a_playback.rs` and `the_end_of_a_playback_of_th07.rs`: `resume::stage_begun` is reached
//! through `Hooks::stage_begun` and the bound, the buttons and the frame they are held against are the same
//! three fields. So the session that showed this was a session of th07 and nothing about it was th07's.
//!
//! **What this game has no counterpart for is the half read off the music.** A fed frame is one the song is
//! held quiet on, so a stretch of fed frames is heard as well as seen — and `Th06c::music` answers `None` on
//! every frame, this game's own streaming being a block a restore leaves alone. So what is left is the half
//! read off the frames.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// How long a stage of this laid-out game lasts here, so that the run picked up in the first walks into the
/// second without a test sitting through a whole one.
const A_STAGE: u32 = 900;

/// How many frames of the stage the run walked into are watched for a frame being fed: past the number the
/// bound left standing would have covered, which is every frame of it below the chapter's own 400.
const WATCHED_FOR: u32 = 500;

/// A launch played out, the game asked to close by its last step.
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

/// **A run left in its second chapter**, and the files that launch left behind.
fn a_run_left_behind() -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 18000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = Fake::attach_configured(
        Display::agreed(120),
        "th06c-the-end-of-a-playback-left",
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.stages_last(A_STAGE);
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
fn a_playback_of_th06c_that_has_landed_feeds_no_frame_of_the_stage_after_it() {
    in_its_own_process(|| {
        let files = a_run_left_behind();
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 36000
- wait_until: {{ frames: 600 }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = Fake::attach_finding(
            Display::agreed(120),
            "th06c-the-end-of-a-playback",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(recipe);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        game.stages_last(A_STAGE);
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
