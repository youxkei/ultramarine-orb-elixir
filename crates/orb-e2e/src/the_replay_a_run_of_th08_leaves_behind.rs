//! **What a Pointdevice run of th08 leaves for somebody to watch: every stretch it played, the one a death
//! rewound it from included.**
//!
//! What `the_replay_a_run_of_th07_leaves_behind.rs` asks of th07, asked of this game. The replay is orb's own
//! and one shape for every game it rewinds, so what is this game's is the run it is written out of: a stage's
//! frames counted from its own build, the chapters of the stage's midstage table, and the slot `Th08::run_slot`
//! names.
//!
//! **Played by a recipe rather than pressed**, the way every other e2e test here that plays a run is: the
//! laid-out stage fires at the player, so a run nobody dodges for is one its bullets decide the chapters of.
//! The recipe is the real game's `a_death_answered_by_the_chapter_again` cut short — into the stage's second
//! chapter, hit there on purpose, the chapter again, and given up the moment the third comes up — so the file
//! holds one death, where that recipe plays on and draws a hit for every chapter after it.
//!
//! **A run still being played is not a saved replay**, and the extension is what says so: `.pending` is the
//! run somebody is in the middle of and `.msgpack` is one that has been kept.

use orb_config::LogLevel;
use orb_core::game::th08::Th08;

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the two kinds.
///
/// Written out here rather than taken from `replay`: what an e2e test says orb left behind is a file at a
/// path, and one that borrowed orb's own idea of that path could not fail if orb changed it.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name orb gives the run these play — Normal, and the first team — which is what names its file.
/// Written out rather than asked of `run_slot`, which would be this agreeing with the code it is about.
const SLOT: &str = "normal-0-ply00a";

/// The date the simulated host's calendar is set to, as a kept replay's name carries it — see
/// `orb_sim::clock`.
const DATED: &str = "20040815-214233";

/// How long a stage of the laid-out game lasts here: past stage 1's midstage table, whose last boundary —
/// the third chapter's start — is at script 4086.
const A_STAGE: u32 = 4_500;

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 40_000;

/// **Into the stage's second chapter, hit there once, and given up as its third comes up** — after the walk
/// every case of this game's makes, `fake::th08::TO_A_RUN`, the part of `a_death_answered_by_the_chapter_again`
/// that has one death in it.
const A_RUN_REWOUND_ONCE: &str = "\
- play_until: { stage: 1, chapter: 2 }
  give_up_after: 3600
- take_hits: { chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }
- play_until: { stage: 1, chapter: 3 }
  give_up_after: 18000
- leave: true
- wait_until: { frames: 300 }
- quit: true
";

/// That run's recipe whole: the walk and then [`A_RUN_REWOUND_ONCE`].
fn a_run_rewound_once_whole() -> String {
    format!("{TO_A_RUN}{A_RUN_REWOUND_ONCE}")
}

/// A launch of this game played by `recipe`, over a stage of the laid-out game's own.
fn launched(name: &str, recipe: &str) -> Box<Fake> {
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

/// **The run rewound once, played out**: answers the frame of the stage the death was on, which is the
/// frame the game stands on while orb's retry question is up — the question freezes it there.
fn a_run_rewound_once(game: &Fake) -> u32 {
    let log = game.log();
    game.frames_until("the death in the second chapter", FRAMES, || {
        log.said("died in chapter 2")
    });
    let died_at = game.state().stage_frames;
    runs_out(game);
    died_at
}

/// Every stretch the run played is in the file, the one that was rewound away included — which is the whole
/// of why orb keeps a replay of its own rather than letting the game keep one.
#[test]
fn a_run_of_th08_writes_down_the_attempt_it_was_rewound_from_as_well_as_the_one_it_went_on_with() {
    in_its_own_process(|| {
        let game = launched("th08-replay-left-behind", &a_run_rewound_once_whole());
        let log = game.log();
        let died_at = a_run_rewound_once(&game);

        // The file is not the frame's to write, so what a test waits for is the writer's own line — the one
        // that says the whole run is in it, which the run being given up is what hands over.
        game.frames_until_a_thread("the whole run written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 3 chapter(s), 4 attempt(s) written"
            ))
        });
        let pending = game
            .dir()
            .join(KEPT_IN)
            .join(format!("{SLOT}.{WHILE_PLAYING}"));
        assert!(
            game.sim().files().holds(&pending),
            "the run being played left no replay at {}:\n  {}",
            pending.display(),
            log.lines().join("\n  ")
        );
        // And nothing offered to watch yet: the run was given up rather than finished with, so its replay is
        // the one still being written and not one of the saved.
        assert!(
            orb_core::replay::saved(game.dir()).is_empty(),
            "a run that was given up was offered as a saved replay",
        );
        assert_eq!(
            orb_core::replay::being_played(game.dir()),
            vec![SLOT.to_owned()]
        );

        let replay = orb_core::replay::read_being_played(game.dir(), SLOT, &Th08)
            .expect("the replay of the run that was given up");
        let stage = match replay.stages.as_slice() {
            [stage] => stage,
            stages => panic!(
                "the run played one stage and the file holds {}",
                stages.len()
            ),
        };
        assert_eq!(stage.stage, 0);
        let numbered: Vec<u32> = stage
            .chapters
            .iter()
            .map(|chapter| chapter.number)
            .collect();
        assert_eq!(
            numbered,
            [1, 2, 3],
            "the chapters the run passed through are not the ones written down, once each",
        );
        let at: Vec<u32> = stage.chapters.iter().map(|chapter| chapter.at).collect();
        assert!(
            at.windows(2).all(|pair| pair[0] < pair[1]),
            "the chapters are not written down in the order the stage played them: {at:?}",
        );

        // The second chapter was played twice — the attempt that was rewound away and the one that went on —
        // and every other chapter once.
        let tries: Vec<usize> = stage
            .chapters
            .iter()
            .map(|chapter| stage.tries(chapter))
            .collect();
        assert_eq!(
            tries,
            [1, 2, 1],
            "the chapter that was retried is not the one with two attempts at it",
        );

        // And what the attempt that was rewound away holds: the frames from the chapter it began at to the
        // frame the death was on, one entry apiece.
        let lost = stage
            .attempt(&stage.chapters[1], 0)
            .expect("the attempt that was rewound away");
        assert_eq!(lost.at, at[1]);
        assert_eq!(
            lost.buttons.len() as u32,
            died_at - at[1],
            "the attempt does not hold the frames between the chapter and the death",
        );
        // And the one that went on holds the frames from that chapter to the frame the chapter after it began
        // on: an attempt at a chapter is that chapter, and the frames past it are the next one's.
        let went_on = stage
            .attempt(&stage.chapters[1], 1)
            .expect("the attempt that went on");
        assert_eq!(went_on.at, at[1]);
        assert_eq!(
            went_on.buttons.len() as u32,
            at[2] - at[1],
            "the attempt that went on does not stop where the chapter it was made at ended",
        );
    });
}

/// **はじめから, which is the moment the run left unfinished is written over.** The replay of the run being
/// written over is kept as the replay of what was played up to there, since nothing after it will ever be —
/// and this game keeps it without a question of its own, the resume question being asked as the run is built.
#[test]
fn starting_a_run_of_th08_over_keeps_the_replay_of_the_run_it_writes_over() {
    in_its_own_process(|| {
        let first = launched(
            "th08-replay-started-over-first",
            &a_run_rewound_once_whole(),
        );
        a_run_rewound_once(&first);
        let left = orb_core::replay::read_being_played(first.dir(), SLOT, &Th08)
            .expect("the replay of the run that was given up");
        let carried: Vec<_> = {
            let files = first.sim().files();
            files
                .paths()
                .into_iter()
                .filter_map(|path| {
                    Some((
                        path.strip_prefix(first.dir()).ok()?.to_path_buf(),
                        files.get(&path)?,
                    ))
                })
                .collect()
        };
        // Closed before the next is launched, as a launch of the real game is: the thread orb writes on is
        // the launch's own — see `fake::th08`'s `Drop`.
        drop(first);

        let second = launched(
            "th08-replay-started-over",
            include_str!(
                "../../orb-real/recipes/th08/the_run_started_over_and_its_replay_kept.yaml"
            ),
        );
        for (inside, bytes) in &carried {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        let log = second.log();
        let kept = format!("{SLOT}-{DATED}");
        second.frames_until_a_thread("the replay kept", || {
            log.said(&format!("replay: kept as {KEPT_IN}/{kept}.{ONCE_KEPT}"))
        });
        assert!(
            log.said("it is kept without asking"),
            "the replay was kept behind a question this game's front end has no press for:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            orb_core::replay::saved(second.dir()),
            vec![kept.clone()],
            "the replay of the run that was written over was not kept:\n  {}",
            log.lines().join("\n  ")
        );
        let saved =
            orb_core::replay::read_saved(second.dir(), &kept, &Th08).expect("the replay kept");
        assert_eq!(
            saved.stages.len(),
            left.stages.len(),
            "what was kept is not what the run had played",
        );
        assert_eq!(
            saved.stages[0].tries(&saved.stages[0].chapters[1]),
            2,
            "the attempts the run made are not in what was kept",
        );

        // And this run writes into a replay of its own, the one it wrote over having been kept — which the
        // recipe plays into the stage's second chapter before it gives this one up too. Waited for by the
        // writer's own line, the file being a thread's of orb's to write and not the frame's.
        runs_out(&second);
        second.frames_until_a_thread("this run's own replay written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 2 chapter(s), 2 attempt(s) written"
            ))
        });
        let fresh = orb_core::replay::read_being_played(second.dir(), SLOT, &Th08)
            .expect("the replay of the run started over");
        let numbered: Vec<u32> = fresh.stages[0]
            .chapters
            .iter()
            .map(|chapter| chapter.number)
            .collect();
        assert_eq!(
            numbered,
            [1, 2],
            "the run started over did not write the chapters it played into a replay of its own",
        );
        // The one kept is left alone by it: a saved replay is not something a later run writes into.
        assert_eq!(orb_core::replay::saved(second.dir()), vec![kept]);
    });
}
