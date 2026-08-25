//! **What a pointdevice run leaves for somebody to watch: every stretch it played, the ones that were
//! rewound away included.**
//!
//! The game's own replay cannot hold this run. Its record of inputs lives in the heap a snapshot covers, so
//! a retry rewinds the record with the run and each attempt writes over the last — what the game would save
//! is the path that survived, with nothing in it of the attempts that did not. That is what *Pointdevice and
//! normal* has always said, and it is why the game's own offer is written past.
//!
//! So orb keeps its own. One file per run there can be one of, beside the chapter that run was left in and
//! named for the same slot — the two correspond, which is the whole of how a run picked up in a later launch
//! goes on being written into the replay it was already being written into. It grows as the run does: a
//! chapter beginning and an attempt being rewound each hand it to the thread that writes orb's files, so
//! whatever ends the session leaves the run watchable up to there.
//!
//! **A run still being played is not a saved replay**, and the extension is what says so: `.pending` is the
//! run somebody is in the middle of, `.msgpack` is one that has been kept. So the list orb offers replays
//! from holds the ones that were finished with, and the run in progress is not in it.
//!
//! What this reads back is the file, through `replay`'s own reader — the same reader the list orb offers
//! them from uses, so a file this says holds two tries of a chapter is a file that offers two tries of it.

use crate::fake::th06::{A_BOMB_IS_TAKEN_FOR, ATTACK_CHANGES, CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::Screen;
use orb_sim::keys;

/// Where orb keeps them, and what it calls the two kinds.
///
/// Written out here rather than taken from `replay`: what an e2e test says orb left behind is a file at a
/// path, and one that borrowed orb's own idea of that path could not fail if orb changed it.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name the game gives the run these e2e tests play: Normal, Reimu A.
const SLOT: &str = "normal-reimu-a";

/// The date the simulated host's calendar is set to, as a saved replay's name carries it — see
/// `orb_sim::clock`.
const DATED: &str = "20040815-214233";

/// A launch whose runs' replays are kept without a question, which is what the setting asks about: what the
/// question is and where it is asked is `the_replay_offered.rs`'s.
fn keeping_every_replay(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        config.save_replays = true;
        config.player_name = "ゆうか".to_owned();
    })
}

/// The run played into its midboss's card, hit there, the chapter retried, and then played on to the
/// chapter after it — which is one chapter with two attempts at it and three with one apiece.
///
/// Returns the frame the attempt that was rewound away ended on, which is what says how long it was: the
/// game is frozen on the retry menu, so the frame it is standing on is the frame the death happened on.
fn a_run_with_a_chapter_played_twice(game: &Fake) -> u32 {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!(
            "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
        ))
    });
    game.hit();
    game.frames_until(
        "the death the hit stands for",
        A_BOMB_IS_TAKEN_FOR as u32 + 2,
        || log.said("died in chapter 3"),
    );
    let died_at = game.state().stage_frames;
    game.press_until(keys::Z, "the chapter retried", || {
        log.said("retry: the chapter again on the keyboard")
    });
    game.frames_until("the chapter after the card", ATTACK_CHANGES + 400, || {
        log.said(&format!("chapter 4 at frame {ATTACK_CHANGES}"))
    });
    died_at
}

/// And that run given up at the retry menu's タイトルに戻る, which is the way out that leaves both files
/// behind: the run did not finish, so nothing takes either of them away.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
}

/// Every stretch the run played is in the file, the one that was rewound away included — which is the whole
/// of why orb keeps a replay of its own rather than letting the game keep one.
#[test]
fn a_run_writes_down_the_attempts_it_was_rewound_from_as_well_as_the_one_that_survived() {
    in_its_own_process(|| {
        let game = keeping_every_replay("replay-left-behind");
        let log = game.log();
        let died_at = a_run_with_a_chapter_played_twice(&game);
        gives_the_run_up(&game);

        // The file is not the frame's to write, so what a test waits for is the writer's own line — and the
        // line that says the whole run is in it, which is the write a run ending is what hands over: on the
        // frame a chapter begins the run has played none of that chapter yet, so the newest one's own
        // attempt is nought frames long until something writes again.
        let pending = game
            .dir()
            .join(KEPT_IN)
            .join(format!("{SLOT}.{WHILE_PLAYING}"));
        game.frames_until_a_thread("the whole run written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 4 chapter(s), 5 attempt(s) written"
            ))
        });
        assert!(
            game.sim().files().holds(&pending),
            "the run being played left no replay at {}:\n  {}",
            pending.display(),
            log.lines().join("\n  ")
        );
        // And nothing is offered to watch yet: the run was left rather than finished with, so its replay is
        // the one still being written and not one of the saved.
        assert!(
            orb_core::replay::saved(game.dir()).is_empty(),
            "a run that was left behind was offered as a saved replay",
        );
        assert_eq!(
            orb_core::replay::being_played(game.dir()),
            vec![SLOT.to_owned()]
        );

        let replay = orb_core::replay::read_being_played(game.dir(), SLOT, &Th06)
            .expect("the replay of the run that was left");
        // Whose run it is, which is what the list shows against it: the name from `orb.yaml`, and the run
        // the game's own screens settled.
        assert_eq!(replay.name, "ゆうか");
        assert_eq!(replay.run.difficulty, the_run().difficulty);
        assert_eq!(replay.run.character, the_run().character);
        assert_eq!(replay.run.shot_type, the_run().shot_type);

        // One stage, since that is as far as the run got, and its chapters in the order the stage plays
        // them.
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
            [1, 2, 3, 4],
            "the chapters the run passed through are not the ones written down, once each",
        );
        // Each at the frame the chapter model settled it on. The first is not the stage's own frame nought:
        // a stage's first chapter waits for the stage to settle and for the music, and where that lands is
        // that model's own — what this asks of it is that it is one chapter and before the second.
        let at: Vec<u32> = stage.chapters.iter().map(|chapter| chapter.at).collect();
        assert_eq!(at[1..], [400, CARD_STARTS, ATTACK_CHANGES]);
        assert!(
            at[0] < 400,
            "the stage's first chapter is at frame {}",
            at[0]
        );

        // The card's chapter was played twice: the attempt that was rewound away, and the one that
        // survived. Every other chapter was played once.
        let tries: Vec<usize> = stage
            .chapters
            .iter()
            .map(|chapter| stage.tries(chapter))
            .collect();
        assert_eq!(
            tries,
            [1, 1, 2, 1],
            "the chapter that was retried is not the one with two attempts at it",
        );

        // And what the attempt that was rewound away holds: the frames from the chapter it began at to the
        // frame the death was on, one entry apiece.
        let lost = stage
            .attempt(&stage.chapters[2], 0)
            .expect("the attempt that was rewound away");
        assert_eq!(lost.at, CARD_STARTS);
        assert_eq!(
            lost.buttons.len() as u32,
            died_at - CARD_STARTS,
            "the attempt does not hold the frames between the chapter and the death",
        );
        // And the one that survived holds the frames from that chapter to the frame the chapter after it
        // began on: an attempt at a chapter is that chapter, and the frames past it are the next one's.
        let survived = stage
            .attempt(&stage.chapters[2], 1)
            .expect("the attempt that survived");
        assert_eq!(survived.at, CARD_STARTS);
        assert_eq!(
            survived.buttons.len() as u32,
            ATTACK_CHANGES - CARD_STARTS,
            "the attempt that survived does not stop where the chapter it was made at ended",
        );
    });
}

/// **はじめから, which is the moment the run left unfinished is written over.** The chapter file is left for
/// this run's own first chapter to write over — and the replay of the run being written over is kept as the
/// replay of what was played up to there, since nothing after it will ever be.
#[test]
fn starting_a_run_over_keeps_the_replay_of_the_run_it_writes_over() {
    in_its_own_process(|| {
        let game = keeping_every_replay("replay-started-over");
        let log = game.log();
        a_run_with_a_chapter_played_twice(&game);
        gives_the_run_up(&game);
        game.frames_until_a_thread("the whole run written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 4 chapter(s), 5 attempt(s) written"
            ))
        });
        let left = orb_core::replay::read_being_played(game.dir(), SLOT, &Th06)
            .expect("the replay of the run that was left");

        // The same run started again, and はじめから answered: one press down from つづきから, which is
        // where the cursor starts.
        game.frames_until("the title menu ready to act on a press", 300, || {
            let front = game.image().front_end_now();
            front.screen == Screen::Title && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered again", || {
            game.image().front_end_now().screen == Screen::ShotType
        });
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said("resume: from the beginning, answered on the keyboard")
        });

        // The replay of the run being written over is kept, under a name carrying the day and the time it
        // was kept at: two runs of one slot are two replays, and what tells them apart is when.
        let kept = format!("{SLOT}-{DATED}");
        game.frames_until_a_thread("the replay kept", || {
            log.said(&format!("replay: kept as {KEPT_IN}/{kept}.{ONCE_KEPT}"))
        });
        assert_eq!(
            orb_core::replay::saved(game.dir()),
            vec![kept.clone()],
            "the replay of the run that was written over was not kept:\n  {}",
            log.lines().join("\n  ")
        );
        let saved =
            orb_core::replay::read_saved(game.dir(), &kept, &Th06).expect("the replay kept");
        assert_eq!(
            saved.stages.len(),
            left.stages.len(),
            "what was kept is not what the run had played",
        );
        assert_eq!(
            saved.stages[0].tries(&saved.stages[0].chapters[2]),
            2,
            "the attempts the run made are not in what was kept",
        );

        // And this run writes into a replay of its own, the one it wrote over having been kept: the file
        // being played into is the run's, and there is exactly one of those per slot.
        game.frames_until("this run's own first chapter", 400, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        game.frames_until_a_thread("this run's own replay written down", || {
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th06)
                .is_some_and(|replay| replay.stages[0].chapters.len() == 1)
        });
        // The one kept is left alone by it: a saved replay is not something a later run writes into.
        assert_eq!(orb_core::replay::saved(game.dir()), vec![kept]);
    });
}
