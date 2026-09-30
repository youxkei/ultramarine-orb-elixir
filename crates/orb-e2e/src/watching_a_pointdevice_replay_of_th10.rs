//! **A replay of a Pointdevice run of th10 kept and watched again — the attempt a death rewound it from.**
//!
//! What the machinery is and why it exists is `watching_a_pointdevice_replay.rs`; all of it is one shape for
//! every game orb rewinds, the record and the file included. **What is this game's own is the run the attempt
//! is watched in**: chosen out of a file rather than on the game's own screens, so orb writes the numbers
//! those screens write and asks the supervisor for the run's scene, and the stage is built on the loader's
//! thread before a frame of the attempt can be played into it.
//!
//! **The attempt watched is the one a retry threw away**, which is what the game's own replay never holds and
//! the one stretch of a run nothing but orb's replay keeps: the run is hit once in its first chapter, so that
//! chapter holds two attempts and the first of them ends on the death.
//!
//! **Three launches, each a case the real game is tested with, each by that case's own recipe**: the death
//! answered by the chapter again, the run started over with its replay kept, and the replay watched — the
//! last answering the lists down to that chapter's first attempt.

use std::path::PathBuf;

use orb_config::LogLevel;
use orb_core::game::th10::Th10;

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run this plays — Normal, and Reimu with her first shot — which names both of its
/// files.
const SLOT: &str = "normal-reimua";
/// The date the simulated host's calendar is set to, which a kept replay's name carries.
const DATED: &str = "20040815-214233";

/// How long a stage of the laid-out game lasts here: past its midboss's two attacks, from frame 1200, which are
/// the stage's second and third chapters.
const A_STAGE: u32 = 4_500;

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 40_000;

/// A launch of this game played by `recipe`, over a stage of the laid-out game's own, with what the launch
/// before it left put into its directory.
fn launched(name: &str, recipe: &str, left: &[(PathBuf, Vec<u8>)]) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    for (inside, bytes) in left {
        game.sim().files().put(game.dir().join(inside), bytes);
    }
    game
}

/// **Played until its recipe quits it**, and failed where it never does.
fn runs_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the launch never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  "),
    );
}

/// What a launch left under its directory, re-rooted under it.
fn left_by(game: &Fake) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            Some((
                path.strip_prefix(game.dir()).ok()?.to_path_buf(),
                files.get(&path)?,
            ))
        })
        .collect()
}

/// **The whole of it over a run of th10**: the run recorded with the attempt it was rewound from, the replay
/// kept where the run left unfinished is written over, and that attempt watched out of the lists.
#[test]
fn the_attempt_a_run_of_th10_was_rewound_from_is_chosen_out_of_its_kept_replay_and_watched() {
    in_its_own_process(|| {
        // ── 1. The run hit in its first chapter and answered by it again, and the frame of the stage its
        // death was on — the frame the game stands on while orb's retry question is up.
        let first = launched(
            "th10-watching-the-run",
            include_str!("../../orb-real/recipes/th10/a_death_answered_by_the_chapter_again.yaml"),
            &[],
        );
        let log = first.log();
        first.frames_until("the death in the first chapter", FRAMES, || {
            log.said("died in chapter 1")
        });
        let died_at = first.state().stage_frames;
        runs_out(&first);
        // Each launch closed before the next, as a launch of the real game is: the thread orb writes on is
        // the launch's own — see `fake::th10`'s `Drop`.
        let left = left_by(&first);
        drop(first);

        // ── 2. はじめから, which keeps the replay of the run it writes over.
        let second = launched(
            "th10-watching-the-run-started-over",
            include_str!(
                "../../orb-real/recipes/th10/the_run_started_over_and_its_replay_kept.yaml"
            ),
            &left,
        );
        runs_out(&second);
        // And the replay that run writes of its own, which is the one being played that the watching below
        // must leave alone — waited for by the writer's own line, the file being a thread's of orb's to
        // write and not the frame's.
        second.frames_until_a_thread("the run started over written down", || {
            second.log().said(&format!(
                "replay: {SLOT}, 1 stage(s), 2 chapter(s), 2 attempt(s) written"
            ))
        });
        let kept = format!("{SLOT}-{DATED}");
        assert_eq!(
            orb_core::replay::saved(second.dir()),
            vec![kept.clone()],
            "the replay of the run that was rewound was not kept:\n  {}",
            second.log().lines().join("\n  ")
        );
        let replay =
            orb_core::replay::read_saved(second.dir(), &kept, &Th10).expect("the replay kept");
        let the_first_chapter = &replay.stages[0].chapters[0];
        assert_eq!(
            replay.stages[0].tries(the_first_chapter),
            2,
            "the chapter the run died in is not the one with two attempts at it",
        );
        let begun_at = the_first_chapter.at;
        let left = left_by(&second);
        drop(second);

        // ── 3. The replay watched, by the case's own recipe: the main menu's item 3, the mode, and the lists
        // answered down to the first chapter's first attempt.
        let third = launched(
            "th10-watching-the-attempt",
            include_str!("../../orb-real/recipes/th10/an_attempt_of_the_kept_replay_watched.yaml"),
            &left,
        );
        let log = third.log();
        let chapter_file = orb_core::resume::left(third.dir());
        let being_played = orb_core::replay::read_being_played(third.dir(), SLOT, &Th10);
        assert!(
            being_played.is_some() && chapter_file == vec![SLOT.to_owned()],
            "the run started over left nothing for the watching to leave alone",
        );
        third.frames_until("the attempt chosen", FRAMES, || {
            log.said("replay: watching attempt")
        });
        let watching = format!(
            "replay: watching attempt 1 of 2 at chapter 1 of stage 1, {} frame(s) of it",
            died_at - begun_at
        );
        assert!(
            log.said(&watching),
            "the attempt watched is not the one the death ended, whose frames are the chapter's to the \
             death's — wanted `{watching}`:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("replay: "))
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        // ── 4. And it is watched a frame at a time, with nothing of the run somebody is in the middle of
        // touched by any of it.
        third.frames_until("the attempt being played", FRAMES, || {
            third.state().stage_frames > begun_at
        });
        let playing = third.state().stage_frames;
        for _ in 0..10 {
            third.frame();
        }
        assert_eq!(
            third.state().stage_frames,
            playing + 10,
            "the attempt is not being played a frame at a time",
        );
        assert_eq!(
            orb_core::resume::left(third.dir()),
            chapter_file,
            "watching a replay wrote over the chapter a run was left in",
        );
        assert_eq!(
            orb_core::replay::read_being_played(third.dir(), SLOT, &Th10),
            being_played,
            "watching a replay wrote over the replay of the run being played",
        );

        // ── 5. The attempt runs out on the frame the death was on, and orb takes the game out of the run it
        // built for it and puts the list back.
        third.frames_until("the attempt over", FRAMES, || {
            log.said("replay: the attempt is over; the run it was watched in is left")
        });
        let over_at = third.state().stage_frames;
        assert!(
            over_at > died_at,
            "the attempt was over at frame {over_at} of the stage, short of the {died_at} it ran to",
        );
        third.frames_until("the list back", FRAMES, || {
            log.said("replay: the attempts at that chapter again")
        });
        runs_out(&third);
    });
}
