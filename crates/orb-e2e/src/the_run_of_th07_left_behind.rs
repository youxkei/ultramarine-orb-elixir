//! **The other answers to どこから始める over a run of th07, and the ways the file behind it is declined.**
//!
//! `the_run_of_th07_written_down_off_the_frame.rs` answers つづきから, which is the one that puts a run back.
//! The two answers it leaves are the ones that do not: **はじめから** starts the run the game was going to start
//! and leaves the file where it is for that run's own first chapter to write over, and a **cancel** starts no
//! run at all — the press that would have started one was held back for the question, so cancelling is that
//! press never being handed over.
//!
//! And the file. A launch reads one per run there can be one of, and three things can be wrong with it that are
//! not "there is none": it cannot be read, it cannot be made sense of, or it holds another run. None of the
//! three is a fault to stop over — the answer is the same as no file at all, which is a run that starts where
//! the game was going to start it — so what each has to do is say which of the three it was and leave the file
//! alone. **Left alone**, because a file that cannot be read is worth looking at and orb deleting it is what
//! would stop anybody doing so.
//!
//! Then the two ways the file cannot be *written*: the directory it goes in cannot be made, and the write
//! itself fails. Both are declared, because these files go through the file seam like every other host call —
//! see [docs/adr/0012](../../../docs/adr/0012-orb-reads-and-writes-its-own-files-through-the-seam.md).
//!
//! Last are the three ways the stage a resume asked for is not what came up: another stage of the same run, a
//! run orb keeps nothing of, and no stage at all. What the first two protect is the one thing about a resume
//! nothing else can catch — the numbers orb holds were read as *that* stage began, so writing them over another
//! stage is a run put back wrong with nothing about the file saying so — and the third is the bound on waiting
//! for a moment that is never coming.

use crate::fake::th07::{Fake, SHOT};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps the runs left unfinished, and what it calls one.
///
/// Written out here rather than taken from `resume`, where both are private: what an e2e test says it left
/// behind is a file at a path, and an e2e test that borrowed orb's own idea of that path could not fail if orb
/// changed it.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which is what names its file: the difficulty and the shot, and nothing
/// else — see `Th07::run_slot`.
///
/// **Written out rather than asked of `run_slot`**, which would be this agreeing with the code it is about: what
/// the name has to be is what somebody reading the directory sees, and `DIFFICULTY`, `CHARACTER` and `SHOT` are
/// 1, 0 and 0 — Normal, and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// And the same difficulty and character with the other shot, which is a run of its own with a file of its own
/// — the buttons written down are the shot's, and another shot's played back would be somebody else's run.
const ANOTHER_SHOTS_SLOT: &str = "normal-reimu-b";

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            // Every replay kept without asking, because these e2e tests are about the *chapter* file: a run
            // played and given up leaves a replay of itself as well, and はじめから is one of the two moments
            // orb offers to keep one — see `watching_a_pointdevice_replay_of_th07`, which drives that offer.
            // Left to be asked, the question would go up over the shot select here and the run would never
            // start.
            config.save_replays = true;
        },
    )
}

/// A launch that plays a run to its stage's own first chapter and then gives it up, which leaves that chapter
/// written down and the game standing at its title menu.
fn left_a_run(name: &str) -> Box<Fake> {
    let game = launched(name);
    let log = game.log();
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames_until_a_thread("the chapter written down", || {
        log.said("chapter 1 (MIDSTAGE 1) at frame")
    });
    game.gives_the_run_up();
    assert_eq!(
        orb_core::resume::left(game.dir()),
        vec![SLOT.to_owned()],
        "the run that was given up was not left behind",
    );
    game
}

/// The same run started again, up to the question and no further.
///
/// `Fake::picks_the_run_up` is this walk with つづきから answered at the end of it; what these e2e tests are
/// about is the answer, so they stop where it is asked.
fn up_to_the_question(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.at_the_shot_select();
    game.press_until(keys::Z, "the question about the run left behind", || {
        log.said_since(
            from,
            &format!("resume: {SLOT} was left; asking where to start"),
        )
    });
}

/// And a run started up to the press at the shot select, waiting for what orb said about the file behind that
/// screen: the file is read where the press is, so nothing is said before one.
fn the_press_at_the_shot_select_says(game: &Fake, said: &str) {
    let log = game.log();
    let from = log.written();
    game.up_to_the_shot_select();
    game.press_until(keys::Z, said, || log.said_since(from, said));
}

/// はじめから: the run the game was going to start, with the file left standing until this run's own first
/// chapter writes over it.
#[test]
fn starting_a_run_of_th07_from_the_beginning_leaves_the_file_for_its_own_first_chapter() {
    in_its_own_process(|| {
        let game = left_a_run("th07-left-from-the-beginning");
        let log = game.log();
        up_to_the_question(&game);

        // One press down from つづきから, after the frames the question reads nothing over: the cursor starts on
        // the answer that costs nothing, which is the run coming back.
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said_since(from, "resume: from the beginning, answered on the keyboard")
        });
        assert!(
            log.said_since(from, "is left behind"),
            "the answer did not say what became of the run it declined:\n  {}",
            log.lines().join("\n  ")
        );

        // The run is the game's own, with nothing played back into it.
        game.image().a_run_has_been_chosen();
        game.builds_a_stage();
        game.frames(8);
        assert!(
            !log.said_since(from, "resume: the landing is"),
            "a run started from the beginning had buttons played into it:\n  {}",
            log.lines().join("\n  ")
        );
        let state = game.state();
        assert_eq!(state.deaths, 0, "a death came out of the file it declined");

        // And the file the answer declined is still there, which is what "left behind" means: it goes when this
        // run reaches a chapter of its own and writes over it, not on the answer.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the file was taken away by an answer that only declined it",
        );
        // The stage's own start is this run's first chapter, which is the chapter that goes into the file —
        // where the run picked up would have begun at the frame it was left in.
        game.frames_until("this run's own first chapter", 600, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        game.frames_until_a_thread("this run's own first chapter written down", || {
            log.said_since(from, "chapter 1 (MIDSTAGE 1) at frame")
        });
    });
}

/// And a cancel: no run at all, and the screen the question went up over carrying on.
#[test]
fn cancelling_the_question_of_th07_starts_no_run_and_leaves_the_shot_select_alone() {
    in_its_own_process(|| {
        let game = left_a_run("th07-left-cancelled");
        let log = game.log();
        up_to_the_question(&game);

        // Held down rather than pressed and let go, which is what the key that cancelled really is: it was down
        // when the question went up. **What it must not do is reach the screen underneath** — the shot select
        // reads back as its own cancel and would go to the character select, which is not what somebody
        // answering "neither" about this question asked for.
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.keyboard().set(keys::X, true);
        game.frames_until("the question cancelled", 60, || {
            log.said_since(
                from,
                "resume: neither, answered on the keyboard; no run is started",
            )
        });
        game.frames(30);
        game.keyboard().set(keys::X, false);
        game.frames(30);

        // No run, and the file untouched: the press that would have started one was never handed over.
        assert!(!game.state().playing, "a cancelled question started a run");
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the run left unfinished was disturbed by a question nobody answered",
        );
    });
}

/// A file that will not decode: named, left alone, and the run starts where the game was going to start it.
#[test]
fn a_file_of_th07_that_cannot_be_made_sense_of_is_named_and_left_where_it_is() {
    in_its_own_process(|| {
        let nonsense = "this is not a msgpack of anything";
        let game = launched("th07-left-nonsense");
        let log = game.log();
        let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().put(&path, nonsense);

        the_press_at_the_shot_select_says(&game, "cannot make sense of");
        assert!(
            log.said(&format!(
                "resume: cannot make sense of {}; it is left alone",
                path.display()
            )),
            "the file that would not decode was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // No question asked, and the press went on to start the run: what a file that cannot be read costs is
        // the same as no file at all.
        assert!(
            !log.said("asking where to start"),
            "a file that could not be read was offered:\n  {}",
            log.lines().join("\n  ")
        );
        // And left exactly as it was, byte for byte, because a file nobody can read is one to go and look at.
        // Read before this run reaches a chapter of its own, which is what would write over it.
        assert_eq!(
            game.sim().files().text(&path).as_deref(),
            Some(nonsense),
            "orb wrote over the file it could not read",
        );
    });
}

/// A file that cannot be read at all — here a read the host refuses, which is the way an e2e test says the read
/// fails rather than finding nothing.
#[test]
fn a_file_of_th07_that_cannot_be_read_is_named_and_costs_the_run_nothing() {
    in_its_own_process(|| {
        let game = launched("th07-left-unreadable");
        let log = game.log();
        let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().put(&path, "a run somebody left");
        game.sim().files().refuses_to_read(&path);

        the_press_at_the_shot_select_says(&game, "cannot read");
        assert!(
            log.said(&format!("resume: cannot read {}", path.display())),
            "the file that could not be read was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("asking where to start"),
            "something that could not be read was offered:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And a file holding another run: the one thing about it that can only be wrong if somebody moved it, since
/// the run it names is what named the file.
#[test]
fn a_file_of_th07_holding_another_runs_chapter_is_named_and_left_alone() {
    in_its_own_process(|| {
        // A run of one shot left behind, and its file copied under the other shot's name — which is what moving
        // one by hand does.
        let held = {
            let game = left_a_run("th07-left-another-run");
            let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
            game.sim()
                .files()
                .get(&path)
                .expect("the file the run was left in")
        };
        let game = launched("th07-left-another-run-again");
        let log = game.log();
        let path = game
            .dir()
            .join(KEPT_IN)
            .join(format!("{ANOTHER_SHOTS_SLOT}.{KEPT_AS}"));
        game.sim().files().put(&path, &held);

        // The other shot under the cursor of the shot select, which is the run that file is asked about for.
        let from = log.written();
        game.up_to_the_shot_select_of_the_shot(SHOT + 1);
        game.press_until(keys::Z, "holds a run of", || {
            log.said_since(from, "holds a run of")
        });
        assert!(
            log.said(&format!(
                "resume: {} holds a run of {SLOT}, not {ANOTHER_SHOTS_SLOT}; left alone",
                path.display(),
            )),
            "the file holding another run was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("asking where to start"),
            "a file holding another run was offered:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.sim().files().get(&path),
            Some(held),
            "orb wrote over a file holding a run it declined",
        );
    });
}

/// A directory orb cannot make, because a file of that name is already there: said, and the run plays on with
/// nothing written down for it.
#[test]
fn a_directory_the_file_of_th07_cannot_go_in_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let game = launched("th07-left-no-directory");
        let log = game.log();
        // A file standing where the directory has to go, which is the one thing that makes `create_dir_all`
        // refuse a path it would otherwise make.
        let at = game.dir().join(KEPT_IN);
        game.sim()
            .files()
            .put(&at, "a file where the directory goes");

        // The first chapter of the run is where the write is tried, so getting that far is what asks the
        // question.
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        // Waited for rather than read at once: the write is not the frame's, so the refusal is the writer's own
        // line — see `the_run_of_th07_written_down_off_the_frame`.
        game.frames_until_a_thread("the directory refused out loud", || {
            log.said(&format!("resume: cannot make {}", at.display()))
        });

        // And it cost the run nothing but the chapter not being written down: the chapter itself happened, the
        // game is still being played, and there is nothing to offer a later launch.
        assert!(log.said("stage 1 chapter 1 (stage start)"));
        let from = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            from + 30,
            "the game stopped being played over a file it could not write",
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "something was offered back out of a directory that was never made",
        );
    });
}

/// And a write that fails with the directory there: the same two properties, and the arm below the one above.
#[test]
fn a_file_of_th07_that_cannot_be_written_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let game = launched("th07-left-no-write");
        let log = game.log();
        let at = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().refuses_to_write(&at);

        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames_until_a_thread("the write refused out loud", || {
            log.said(&format!("resume: cannot write {}", at.display()))
        });
        assert!(log.said("stage 1 chapter 1 (stage start)"));
        let from = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            from + 30,
            "the game stopped being played over a file it could not write",
        );
    });
}

/// A stage that comes up where another was asked for: the numbers are **not** written over it, and orb says
/// which stage arrived.
#[test]
fn a_stage_of_th07_other_than_the_one_asked_for_is_not_written_over() {
    in_its_own_process(|| {
        let game = left_a_run("th07-left-another-stage");
        let log = game.log();
        up_to_the_question(&game);
        let from = log.written();
        game.press_until(keys::Z, "つづきから answered", || {
            log.said_since(
                from,
                "resume: from where it stopped, answered on the keyboard",
            )
        });
        game.image().a_run_has_been_chosen();
        game.frames_until("the stage the run was left in asked for", 8, || {
            log.said_since(from, "the run is being built at that stage")
        });

        // The second stage where the first was asked for, counted from zero the way everything above `Game`
        // counts.
        game.builds_the_stage(1);
        game.frames_until("the stage that came up instead", 60, || {
            log.said_since(
                from,
                "resume: not put back — stage 2 came up where stage 1 was asked for",
            )
        });

        // Nothing of the run went in: no buttons were played, which is the only safe answer to a stage nobody
        // asked for.
        assert!(
            !log.said_since(from, "resume: the landing is"),
            "buttons went into a stage that was not the one asked for:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the attract demo coming up instead, which is a run orb keeps nothing of: said as that rather than as the
/// wrong stage, the two being different things to have gone wrong.
#[test]
fn a_run_of_th07_orb_keeps_nothing_of_is_not_written_over_either() {
    in_its_own_process(|| {
        let game = left_a_run("th07-left-a-demo");
        let log = game.log();
        up_to_the_question(&game);
        let from = log.written();
        game.press_until(keys::Z, "つづきから answered", || {
            log.said_since(
                from,
                "resume: from where it stopped, answered on the keyboard",
            )
        });
        game.image().a_run_has_been_chosen();
        game.frames_until("the stage the run was left in asked for", 8, || {
            log.said_since(from, "the run is being built at that stage")
        });

        // The demo, which is the bit the title screen's own idle count sets: the same stage built, and a run
        // nobody is playing.
        game.image().is_the_attract_demo();
        game.builds_a_stage();
        game.frames_until("the run orb keeps nothing of", 60, || {
            log.said_since(
                from,
                "resume: not put back — what came up is a demo, a replay, or a run in a mode that \
                 keeps nothing",
            )
        });
        assert!(
            !log.said_since(from, "resume: the landing is"),
            "buttons went into a run orb keeps nothing of:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the stage a resume asks for that never comes up: waited for, given up on, and the run left running.
#[test]
fn a_stage_of_th07_that_never_comes_up_is_given_up_on() {
    in_its_own_process(|| {
        let game = left_a_run("th07-left-no-stage");
        let log = game.log();
        up_to_the_question(&game);
        let from = log.written();
        game.press_until(keys::Z, "つづきから answered", || {
            log.said_since(
                from,
                "resume: from where it stopped, answered on the keyboard",
            )
        });
        // The run chosen, which is the frame the stage is asked for — and then no build at all, which is a
        // stage the game cannot load.
        game.image().a_run_has_been_chosen();
        game.frames_until("the stage the run was left in asked for", 8, || {
            log.said_since(from, "the run is being built at that stage")
        });
        game.the_stage_never_comes_up();

        // Ten seconds of frames with no stage built, and then given up on: what orb must not do is wait for
        // ever with a run's numbers held for a moment that is never coming.
        game.frames(RESUME_WAIT + 60);
        assert!(
            log.said_since(from, "resume: given up — the stage never came up"),
            "orb is still waiting for a stage that never came:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said_since(from, "resume: the landing is"),
            "buttons went into a stage that never came up:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// How long orb waits for a stage a resume asked for, in frames.
///
/// Ten seconds of them, which is `runtime::RESUME_START_FRAMES` — written out here for the same reason the path
/// above is, that constant being private.
const RESUME_WAIT: u32 = 600;
