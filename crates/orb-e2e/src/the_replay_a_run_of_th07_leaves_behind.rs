//! **What a pointdevice run of th07 leaves for somebody to watch: every stretch it played, the ones that
//! were rewound away included.**
//!
//! What `the_replay_a_run_leaves_behind.rs` asks of th06, asked of this game. The replay is orb's own and
//! one shape for both games — so what is this game's is the run it is written out of: th07's own frame counter, the chapters its
//! fights are divided into, and the slot its front end's three numbers name. A file this reads back as two
//! tries of a chapter is a file the lists offer two tries of, the reader being the same.
//!
//! **A run still being played is not a saved replay**, and the extension is what says so: `.pending` is the
//! run somebody is in the middle of, `.msgpack` is one that has been kept.

use crate::fake::th07::{CHARACTER, DIFFICULTY, Fake, SHOT};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::Th07;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the two kinds.
///
/// Written out here rather than taken from `replay`: what an e2e test says orb left behind is a file at a
/// path, and one that borrowed orb's own idea of that path could not fail if orb changed it.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name orb gives the run these e2e tests play — Normal, and Reimu's first shot — which is what names
/// its file. Written out rather than asked of `run_slot`, which would be this agreeing with the code it is
/// about.
const SLOT: &str = "normal-reimu-a";

/// The date the simulated host's calendar is set to, as a saved replay's name carries it — see
/// `orb_sim::clock`.
const DATED: &str = "20040815-214233";

/// Whose runs these are, as `orb.yaml` says.
const PLAYER: &str = "ゆうか";

/// How long each stretch of the fight this run plays is, which is what the e2e tests about this game's
/// chapters drive it in: past the shortest a chapter may be.
const APART: u32 = 120;

/// The card the midboss of this run's stage runs, and the name the game puts on the plate for it.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// A launch whose runs' replays are kept without a question, which is what the setting asks about: what the
/// question is and where it is asked is `the_replay_offered_of_th07.rs`'s.
fn keeping_every_replay(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
            config.player_name = PLAYER.to_owned();
        },
    );
    game.streams_its_song(0);
    game
}

/// The run played into its midboss's card, hit there, the chapter retried, and then played on to the
/// chapter after it — which is one chapter with two attempts at it and three with one apiece.
///
/// Returns the frame the attempt that was rewound away ended on, which is what says how long it was: the
/// game is frozen on the retry menu, so the frame it is standing on is the frame the death happened on.
fn a_run_with_a_chapter_played_twice(game: &Fake) -> u32 {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames(APART);
    game.the_boss_arrives();
    game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
    game.frames(APART);
    game.a_card_starts(CARD, CARD_NAME);
    game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
    game.frames(APART);
    game.dies();
    game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
    let died_at = game.state().stage_frames;
    let from = log.written();
    game.press_until(keys::Z, "the chapter retried", || {
        log.said_since(from, "retry: the chapter again on the keyboard")
    });
    game.frames(APART);
    game.the_card_is_over();
    game.the_attack_changes();
    game.frames_until("the chapter after the card", APART, || {
        log.said_since(from, "chapter 4")
    });
    died_at
}

/// Every stretch the run played is in the file, the one that was rewound away included — which is the whole
/// of why orb keeps a replay of its own rather than letting the game keep one.
#[test]
fn a_run_of_th07_writes_down_the_attempts_it_was_rewound_from_as_well_as_the_one_that_survived() {
    in_its_own_process(|| {
        let game = keeping_every_replay("th07-replay-left-behind");
        let log = game.log();
        let died_at = a_run_with_a_chapter_played_twice(&game);
        // タイトルに戻る, which is the way out that leaves both files behind: the run did not finish, so
        // nothing takes either of them away.
        game.gives_the_run_up();

        // The file is not the frame's to write, so what a test waits for is the writer's own line — and the
        // line that says the whole run is in it, which is the write a run ending is what hands over.
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

        let replay = orb_core::replay::read_being_played(game.dir(), SLOT, &Th07)
            .expect("the replay of the run that was left");
        // Whose run it is, which is what the list shows against it: the name from `orb.yaml`, and the run
        // the game's own screens settled.
        assert_eq!(replay.name, PLAYER);
        assert_eq!(replay.run.difficulty, DIFFICULTY);
        assert_eq!(replay.run.character, i32::from(CHARACTER));
        assert_eq!(replay.run.shot_type, i32::from(SHOT));

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
        // Each on the frame the chapter model settled it on, in the order the stage played them: the frames
        // themselves are the model's own — the fight is driven from here a stretch at a time — so what this
        // asks of them is that each is after the one before.
        let at: Vec<u32> = stage.chapters.iter().map(|chapter| chapter.at).collect();
        assert!(
            at.windows(2).all(|pair| pair[0] < pair[1]),
            "the chapters are not written down in the order the stage played them: {at:?}",
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
        assert_eq!(lost.at, at[2]);
        assert_eq!(
            lost.buttons.len() as u32,
            died_at - at[2],
            "the attempt does not hold the frames between the chapter and the death",
        );
        // And the one that survived holds the frames from that chapter to the frame the chapter after it
        // began on: an attempt at a chapter is that chapter, and the frames past it are the next one's.
        let survived = stage
            .attempt(&stage.chapters[2], 1)
            .expect("the attempt that survived");
        assert_eq!(survived.at, at[2]);
        assert_eq!(
            survived.buttons.len() as u32,
            at[3] - at[2],
            "the attempt that survived does not stop where the chapter it was made at ended",
        );
    });
}

/// **はじめから, which is the moment the run left unfinished is written over.** The chapter file is left for
/// this run's own first chapter to write over — and the replay of the run being written over is kept as the
/// replay of what was played up to there, since nothing after it will ever be.
#[test]
fn starting_a_run_of_th07_over_keeps_the_replay_of_the_run_it_writes_over() {
    in_its_own_process(|| {
        let game = keeping_every_replay("th07-replay-started-over");
        let log = game.log();
        a_run_with_a_chapter_played_twice(&game);
        game.gives_the_run_up();
        game.frames_until_a_thread("the whole run written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 4 chapter(s), 5 attempt(s) written"
            ))
        });
        let left = orb_core::replay::read_being_played(game.dir(), SLOT, &Th07)
            .expect("the replay of the run that was left");

        // The same run started again, and はじめから answered: one press down from つづきから, which is
        // where the cursor starts. The mode is answered already, a hand that has just given a run up standing
        // at the title menu with it settled — see `Fake::gives_the_run_up`.
        let from = log.written();
        game.at_the_shot_select();
        game.press_until(keys::Z, "the question about the run left behind", || {
            log.said_since(from, "asking where to start")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said_since(from, "resume: from the beginning, answered on the keyboard")
        });

        // The replay of the run being written over is kept, under a name carrying the day and the time it
        // was kept at: two runs of one slot are two replays, and what tells them apart is when.
        let kept = format!("{SLOT}-{DATED}");
        game.frames_until_a_thread("the replay kept", || {
            log.said_since(
                from,
                &format!("replay: kept as {KEPT_IN}/{kept}.{ONCE_KEPT}"),
            )
        });
        assert_eq!(
            orb_core::replay::saved(game.dir()),
            vec![kept.clone()],
            "the replay of the run that was written over was not kept:\n  {}",
            log.lines().join("\n  ")
        );
        let saved =
            orb_core::replay::read_saved(game.dir(), &kept, &Th07).expect("the replay kept");
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
        // being played into is the run's, and there is exactly one of those per slot. The game builds the run
        // the answer let through — the press was handed back to the shot select, whose own decide chooses it.
        game.image().a_run_has_been_chosen();
        game.builds_a_stage();
        game.frames_until("this run's own first chapter", 600, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        game.frames_until_a_thread("this run's own replay written down", || {
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th07)
                .is_some_and(|replay| replay.stages[0].chapters.len() == 1)
        });
        // The one kept is left alone by it: a saved replay is not something a later run writes into.
        assert_eq!(orb_core::replay::saved(game.dir()), vec![kept]);
    });
}
