//! **The other answers to どこから始める, and the ways the file behind it is declined.**
//!
//! `pointdevice_run.rs` answers つづきから, which is the one that puts a run back. The two answers it
//! leaves are the ones that do not: **はじめから** starts the run the game was going to start and leaves
//! the file where it is for that run's own first chapter to write over, and a **cancel** starts no run at
//! all — the press that would have started one was held back for the question, so cancelling is that press
//! never being handed over.
//!
//! And the file. A launch reads one per run there can be one of, and three things can be wrong with it that
//! are not "there is none": it cannot be read, it cannot be made sense of, or it holds another run. None of
//! the three is a fault to stop over — the answer is the same as no file at all, which is a run that starts
//! where the game was going to start it — so what each has to do is say which of the three it was and leave
//! the file alone. **Left alone**, because a file that cannot be read is worth looking at and orb deleting
//! it is what would stop anybody doing so.
//!
//! Then the two ways the file cannot be *written*: the directory it goes in cannot be made, and the write
//! itself fails. **Both are declared** — `orb_sim::Files::refuses_to_write`, and a file put where the
//! directory has to go — because these files go through the file seam like every other host call. Asking
//! the same question of a real disk means arranging one into a shape that makes `std::fs` refuse, which is
//! an e2e test asserting about the machine it happens to be on. See
//! [docs/adr/0012](../../../docs/adr/0012-orb-reads-and-writes-its-own-files-through-the-seam.md).
//!
//! Last is the stage that never comes. A resume points the run the game is about to build at a stage and
//! holds its numbers for the moment that stage is ready for them; a stage the game cannot load is a wait
//! that never ends, so it is bounded, and this is that bound.

use crate::fake::th06::{CARD_STARTS, ComesUpAs, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::RunStart;
use orb_core::game::th06::image::{Scene, Screen};
use orb_sim::keys;

/// Where orb keeps the runs left unfinished, and what it calls one.
///
/// Written out here rather than taken from `resume`, where both are private: what an e2e test says it left
/// behind is a file at a path, and an e2e test that borrowed orb's own idea of that path could not fail if
/// orb changed it.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name the game gives the run this file plays: Normal, Reimu A.
const SLOT: &str = "normal-reimu-a";

/// And the same difficulty and character with the other shot, which is a run of its own with a file of its
/// own — the buttons written down are the shot's, and another shot's played back would be somebody else's
/// run.
const OTHER_SHOT: &str = "normal-reimu-b";

fn the_other_shot() -> RunStart {
    RunStart {
        shot_type: 1,
        ..the_run()
    }
}

/// A launch that plays a run into its midboss's card and then gives it up at the game's own pause, which
/// leaves the chapter written down.
fn left_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!(
            "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
        ))
    });
    // やめる at the game's own pause, which is the one way out of a run that leaves the chapter behind: the
    // run did not finish, so nothing takes the file away.
    game.gives_the_run_up_at_its_own_pause();
    game.frames_until("the front end", 60, || {
        game.image().scene() == Scene::FrontEnd
    });
    assert_eq!(
        orb_core::resume::left(game.dir()),
        vec![SLOT.to_owned()],
        "the run that was given up was not left behind",
    );
    game
}

/// A run started from the title menu as far as the press at the shot type select, which is the frame the
/// file behind the question is read on and the last frame before the run's own first chapter writes over it.
fn started_up_to_the_press(game: &Fake) {
    let log = game.log();
    game.at_the_title_menu();
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered", || {
        log.said("mode: answered on the keyboard")
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
}

/// And the stage that press starts, built.
fn the_stage_is_built(game: &Fake) {
    game.frames_until("the stage built", 90, || {
        let state = game.state();
        state.playing && state.stage_frames >= 1
    });
}

/// The same run started again, up to the question and no further.
///
/// `Fake::picks_the_run_up` is this walk with つづきから answered at the end of it; what these e2e tests are
/// about is the answer, so they stop where it is asked.
fn up_to_the_question(game: &Fake) {
    let log = game.log();
    game.frames_until("the title menu ready to act on a press", 300, || {
        let front = game.image().front_end_now();
        game.image().scene() == Scene::FrontEnd
            && front.screen == Screen::Title
            && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered again", || {
        game.image().front_end_now().screen == Screen::ShotType
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        game.image().front_end_now().acts_on_a_press()
    });
    game.press(keys::Z);
    assert!(
        log.said(&format!("resume: {SLOT} was left; asking where to start")),
        "the question was not asked:\n  {}",
        log.lines().join("\n  ")
    );
}

/// はじめから: the run the game was going to start, with the file left standing until this run's own first
/// chapter writes over it.
#[test]
fn starting_from_the_beginning_leaves_the_file_for_the_new_runs_first_chapter() {
    in_its_own_process(|| {
        let game = left_a_run("left-from-the-beginning");
        let log = game.log();
        up_to_the_question(&game);

        // One press down from つづきから, after the frames the question reads nothing over: the cursor starts
        // on the answer that costs nothing, which is the run coming back.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said("resume: from the beginning, answered on the keyboard")
        });
        assert!(
            log.said("is left behind"),
            "the answer did not say what became of the run it declined:\n  {}",
            log.lines().join("\n  ")
        );

        // The run is the game's own, with nothing played back into it.
        let from = log.lines().len();
        the_stage_is_built(&game);
        assert!(
            !log.said_since(from, "resume: the landing is"),
            "a run started from the beginning had buttons played into it:\n  {}",
            log.lines().join("\n  ")
        );
        let state = game.state();
        assert_eq!(state.deaths, 0);
        assert_eq!(state.lives, 2, "the lives a run starts with");

        // And the file the answer declined is still there, which is what "left behind" means: it goes when
        // this run reaches a chapter of its own and writes over it, not on the answer.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the file was taken away by an answer that only declined it",
        );
        // The stage's own start is this run's first chapter, which is the chapter that goes into the file —
        // where the run picked up would have begun at the card it was left in.
        game.frames_until("this run's own first chapter", 400, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        game.frames_until("this run's own first chapter written down", 60, || {
            log.said_since(from, "chapter 1 (MIDSTAGE 1) at frame")
        });
    });
}

/// And a cancel: no run at all, and the screen the question went up over carrying on.
#[test]
fn cancelling_the_question_starts_no_run_and_leaves_the_shot_select_alone() {
    in_its_own_process(|| {
        let game = left_a_run("left-cancelled");
        let log = game.log();
        up_to_the_question(&game);

        // Held down rather than pressed and let go, which is what the key that cancelled really is: it was
        // down when the question went up. **What it must not do is reach the screen underneath** — the shot
        // type select reads back as its own cancel and would go to the character select, which is not what
        // somebody answering "neither" about this question asked for.
        game.frames(READS_KEYS_AFTER);
        game.keyboard().set(keys::X, true);
        game.frames_until("the question cancelled", 60, || {
            log.said("resume: neither, answered on the keyboard; no run is started")
        });
        game.frames(30);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::ShotType,
            "the key that cancelled the question reached the screen under it",
        );
        game.keyboard().set(keys::X, false);
        game.frames(30);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::ShotType,
            "the screen moved when the key that cancelled was let go",
        );

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
fn a_file_that_cannot_be_made_sense_of_is_named_and_left_where_it_is() {
    in_its_own_process(|| {
        let nonsense = "this is not a msgpack of anything";
        let at = format!("{KEPT_IN}/{SLOT}.{KEPT_AS}");
        let game = Fake::attach_finding("left-nonsense", &[(&at, nonsense)], the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        started_up_to_the_press(&game);

        let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        assert!(
            log.said(&format!(
                "resume: cannot make sense of {}; it is left alone",
                path.display()
            )),
            "the file that would not decode was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // No question asked, and the press went on to start the run: what a file that cannot be read costs
        // is the same as no file at all.
        assert!(
            !log.said("asking where to start"),
            "a file that could not be read was offered:\n  {}",
            log.lines().join("\n  ")
        );
        // And left exactly as it was, byte for byte, because a file nobody can read is one to go and look
        // at. Read before this run reaches a chapter of its own, which is what would write over it.
        assert_eq!(
            game.sim().files().text(&path).as_deref(),
            Some(nonsense),
            "orb wrote over the file it could not read",
        );
        the_stage_is_built(&game);
    });
}

/// A file that cannot be read at all — here a directory standing where the file goes, which is the way an
/// e2e test can say the read fails rather than finding nothing.
#[test]
fn a_file_that_cannot_be_read_is_named_and_costs_the_run_nothing() {
    in_its_own_process(|| {
        let game = Fake::attach("left-unreadable", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        // Declared after the launch and before the run is chosen, which is where the read happens: the
        // launch's own list of runs left unfinished is read at the attach, and this one is asked for at the
        // shot type select.
        let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().put(&path, "a run somebody left");
        game.sim().files().refuses_to_read(&path);

        started_up_to_the_press(&game);
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
        the_stage_is_built(&game);
    });
}

/// And a file holding another run: the one thing about it that can only be wrong if somebody moved it, since
/// the run it names is what named the file.
#[test]
fn a_file_holding_another_runs_chapter_is_named_and_left_alone() {
    in_its_own_process(|| {
        // A run of one shot left behind, and its file copied under the other shot's name — which is what
        // moving one by hand does.
        let held = {
            let game = left_a_run("left-another-run");
            let path = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
            game.sim()
                .files()
                .get(&path)
                .expect("the file the run was left in")
        };
        let at = format!("{KEPT_IN}/{OTHER_SHOT}.{KEPT_AS}");
        let game = Fake::attach_finding(
            "left-another-run-again",
            &[(&at, "")],
            the_other_shot(),
            |config| {
                config.log_level = LogLevel::Verbose;
            },
        );
        let path = game
            .dir()
            .join(KEPT_IN)
            .join(format!("{OTHER_SHOT}.{KEPT_AS}"));
        game.sim().files().put(&path, &held);
        let log = game.log();

        started_up_to_the_press(&game);
        assert!(
            log.said(&format!(
                "resume: {} holds a run of {SLOT}, not {OTHER_SHOT}; left alone",
                path.display()
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
        the_stage_is_built(&game);
    });
}

/// A directory orb cannot make, because a file of that name is already there: said, and the run plays on
/// with nothing written down for it.
#[test]
fn a_directory_the_file_cannot_go_in_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        // A file standing where the directory has to go, which is the one thing that makes `create_dir_all`
        // refuse a path it would otherwise make.
        let game = Fake::attach_finding(
            "left-no-directory",
            &[(KEPT_IN, "a file where the directory goes")],
            the_run(),
            |config| {
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        let at = game.dir().join(KEPT_IN);
        assert!(
            game.sim().files().holds(&at),
            "the file this is about is not there",
        );

        // The first chapter of the run is where the write is tried, so getting that far is what asks the
        // question.
        game.in_a_pointdevice_run();
        assert!(
            log.said(&format!("resume: cannot make {}", at.display())),
            "the directory that could not be made was not named:\n  {}",
            log.lines().join("\n  ")
        );

        // And it cost the run nothing but the chapter not being written down: the chapter itself happened,
        // the game is still being played, and there is nothing to offer a later launch.
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

/// And a write that fails with the directory there, because what the file's own name points at is a
/// directory: the same two properties, and the arm below the one above.
#[test]
fn a_file_that_cannot_be_written_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let game = Fake::attach("left-no-write", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        // Declared after the launch, so that the launch's own read of what was left is the read of a
        // directory with nothing in it — this is about the *write*.
        let at = game.dir().join(KEPT_IN).join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().refuses_to_write(&at);

        game.in_a_pointdevice_run();
        assert!(
            log.said(&format!("resume: cannot write {}", at.display())),
            "the file that could not be written was not named:\n  {}",
            log.lines().join("\n  ")
        );
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
///
/// What is being protected is the one thing about a resume that nothing else can catch. The numbers orb
/// holds were read as *that* stage began — the score, the seed, the lives — so writing them over another
/// stage is a run put back wrong with nothing about the file saying so. Orb compares the stage that arrived
/// with the stage it asked for, and this is what makes there be something to compare.
#[test]
fn a_stage_other_than_the_one_asked_for_is_not_written_over() {
    in_its_own_process(|| {
        let game = left_a_run("left-another-stage");
        let log = game.log();
        // Stage 2 where stage 1 was asked for, counted from zero the way everything above `Game` counts.
        game.comes_up_as(Some(ComesUpAs::AnotherStage(1)));
        up_to_the_question(&game);

        game.press_until(keys::Z, "つづきから answered", || {
            log.said("resume: from where it stopped, answered on the keyboard")
        });
        game.frames_until("the stage that came up instead", 90, || {
            log.said("resume: not put back — stage 2 came up where stage 1 was asked for")
        });

        // Nothing of the run went in: no buttons were played and the run is the game's own from its
        // beginning, which is the only safe answer to a stage nobody asked for.
        assert!(
            !log.said("resume: the landing is"),
            "buttons went into a stage that was not the one asked for:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().deaths,
            0,
            "the run carried a death out of the file it was not put back from",
        );
    });
}

/// And the attract demo coming up instead, which is a run orb keeps nothing of: said as that rather than as
/// the wrong stage, the two being different things to have gone wrong.
#[test]
fn a_run_orb_keeps_nothing_of_is_not_written_over_either() {
    in_its_own_process(|| {
        let game = left_a_run("left-a-demo");
        let log = game.log();
        game.comes_up_as(Some(ComesUpAs::TheDemo));
        up_to_the_question(&game);

        game.press_until(keys::Z, "つづきから answered", || {
            log.said("resume: from where it stopped, answered on the keyboard")
        });
        game.frames_until("the run orb keeps nothing of", 90, || {
            log.said(
                "resume: not put back — what came up is a demo, a replay, or a run in a mode that \
                 keeps nothing",
            )
        });
        assert!(
            !log.said("resume: the landing is"),
            "buttons went into a run orb keeps nothing of:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the stage a resume asks for that never comes up: waited for, given up on, and the run left running.
#[test]
fn a_stage_that_never_comes_up_is_given_up_on() {
    in_its_own_process(|| {
        let game = left_a_run("left-no-stage");
        let log = game.log();
        game.never_builds_the_stage_it_is_asked_for();
        up_to_the_question(&game);

        game.press_until(keys::Z, "つづきから answered", || {
            log.said("resume: from where it stopped, answered on the keyboard")
        });
        // The press the question was answered on is handed back to the shot type select, and it is that
        // screen's own decide that chooses the run — so the stage is asked for a frame or two later.
        game.frames_until("the stage the run was left in asked for", 60, || {
            log.said("the run is being built at that stage")
        });

        // Ten seconds of frames, and then given up on: what orb must not do is wait for ever with a run's
        // numbers held for a moment that is never coming.
        game.frames(RESUME_WAIT + 60);
        assert!(
            log.said("resume: given up — the stage never came up"),
            "orb is still waiting for a stage that never came:\n  {}",
            log.lines().join("\n  ")
        );
        // And nothing of the run was played in, there having been no stage to play it into.
        assert!(
            !log.said("resume: the landing is"),
            "buttons went into a stage that never came up:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// How long orb waits for a stage a resume asked for, in frames.
///
/// Ten seconds of them, which is `runtime::RESUME_START_FRAMES` — written out here for the same reason the
/// path above is, that constant being private.
const RESUME_WAIT: u32 = 600;
