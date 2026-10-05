//! **The answers to どこから始める over a run of th06nc that do not put one back, and the ways the file
//! behind it is declined.**
//!
//! `a_pointdevice_run_of_th06nc.rs` answers つづきから, which is the one that does. The other answer and
//! the ways the file can be wrong are here, and they are the half of the mode nothing else covers for this
//! game: a launch that read a file it should have declined puts a run back wrong, and one that stops over
//! a file it could not read takes a run away from somebody who still has it on disk. th06c's own answer is
//! `the_run_of_th06c_left_behind.rs`, and this is that file over this game's screens.
//!
//! **はじめから** starts the run the game was going to start and leaves the file standing for that run's
//! own first chapter to write over — the file goes when a chapter is written, not on the answer. Three
//! things can be wrong with a file that are not "there is none": it cannot be read, it cannot be made
//! sense of, or it holds another run. None is a fault to stop over, so each has to say which of the three
//! it was and **leave the file where it is**. Then the two ways the file cannot be *written*: the
//! directory it goes in cannot be made, and the write itself fails. Both go through the file seam like
//! every other host call, so both are declared.
//!
//! **And a cancel**, which is the answer that starts no run at all: the press that would have started one
//! was held back for the question, so cancelling is that press never being handed over.
//!
//! **The run is left in its first chapter and not its second**, which is where this differs from th06c's:
//! that game has boundaries inside a stage and this one has none until `th06nc/chapters.rs` is filled, so
//! a stage of th06nc is one chapter and a run left mid-stage is a run left in chapter 1. What the file
//! being read back is therefore held against is the frame the run was left in — see
//! `a_pointdevice_run_of_th06nc`, which plays the same run and asserts it comes back frame for frame.
//!
//! **And the question stands over `front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS`**, screen 24, where
//! th06c's stands over its shot select: this game's walk puts three selects and a mode select between the
//! menu and the run, and 24 is the one screen before a stage that takes a press. See
//! `Th06nc::run_pointed_at`, which reads the run there.

use std::path::{Path, PathBuf};

use orb_core::game::th06nc::image::SCREEN_TITLE;
use orb_sim::keys;

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long a stage of this laid-out game lasts, so that a run left in one is left mid-stage rather than
/// walked out of it — the same number `a_pointdevice_run_of_th06nc` gives it.
const A_STAGE: u32 = 1_200;

/// How long the title is left standing before a press at it, which is what `TO_A_RUN` waits out too: a
/// decide inside those frames is a decide the screen read nothing over.
const A_TITLES_OWN_FRAMES: u32 = 60;

/// Where orb keeps the runs left unfinished, and what it calls one.
///
/// Written out here rather than taken from `resume`, where both are private: what an e2e test says it left
/// behind is a file at a path, and one that borrowed orb's own idea of that path could not fail if orb
/// changed it.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file: Normal, Reimu's first shot, and 標準モード
/// — which `Th06nc::run_slot` leaves out of the name, Challenge being the mode that carries a word.
///
/// Written out rather than asked of `run_slot`, which would be this agreeing with the code it is about.
const SLOT: &str = "normal-reimu-a";

/// And the same difficulty and character with the other shot, which is a run of its own with a file of its
/// own — the buttons written down are the shot's, and another shot's played back would be somebody else's
/// run.
const ANOTHER_SHOTS_SLOT: &str = "normal-reimu-b";

fn launched(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            // Every replay kept without asking, these being about the chapter file: a run given up leaves
            // a replay of itself too, and left to be asked that question would stand where this one wants
            // to.
            config.save_replays = true;
        },
    );
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

/// A run played into its stage and left there, and the files that launch left behind.
fn a_run_left_behind(name: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- quit: true
"
    );
    let game = launched(name, recipe);
    played_out(&game);
    let files = left_by(&game, game.launch().dir());
    assert!(
        files
            .iter()
            .any(|(inside, _)| is_the_runs_file(inside, SLOT)),
        "no chapter file of {SLOT} was left: {:?}",
        files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
    );
    files
}

/// Whether a path under orb's directory is the file a run of `slot` is left in.
fn is_the_runs_file(inside: &Path, slot: &str) -> bool {
    let said = inside.to_string_lossy();
    said.contains(KEPT_IN) && said.contains(slot) && said.ends_with(KEPT_AS)
}

/// A launch that finds `files` and walks to the question, answering it with `item`, then plays on for as
/// long as `after` says and quits.
fn the_answer(name: &str, files: &[(PathBuf, Vec<u8>)], item: usize, after: &str) -> Box<Fake> {
    let recipe = format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: {item} }}
  give_up_after: 600
{after}\
- quit: true
"
    );
    // **Found at the attach and not put in afterwards**: what a launch has to offer it reads as it comes
    // up, so the files go in before orb does.
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.stages_last(A_STAGE);
    game
}

/// はじめから: the run the game was going to start, with the file left standing until this run's own first
/// chapter writes over it.
#[test]
fn starting_a_run_of_th06nc_from_the_beginning_leaves_the_file_for_its_own_first_chapter() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-for-the-beginning");
        let game = the_answer(
            "th06nc-from-the-beginning",
            &files,
            1,
            "\
- press: [decide]
  frames: 6
  until: { stage: 1, chapter: 1 }
  give_up_after: 3600
- wait_until: { frames: 120 }
",
        );
        played_out(&game);
        let log = game.log();

        assert!(
            log.said("resume: from the beginning"),
            "はじめから was answered and nothing said so:\n  {}",
            log.lines().join("\n  ")
        );
        // The run is the game's own: nothing of the file was played into it, which is what the landing
        // being unsaid means.
        assert!(
            !log.said("resume: the landing is"),
            "a run started from the beginning had the file played into it:\n  {}",
            log.lines().join("\n  ")
        );
        // Its first chapter is the stage's own start, where the run picked up would have begun at the
        // frame it was left in.
        assert!(
            log.said("stage 1 chapter 1"),
            "the run did not begin at its own first chapter:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A file that cannot be read at all — a read the host refuses, which is how an e2e test says the read
/// failed without inventing a file that cannot exist.
#[test]
fn a_file_of_th06nc_that_cannot_be_read_is_named_and_costs_the_run_nothing() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-unreadable");
        let game = the_answer(
            "th06nc-unreadable",
            &files,
            0,
            "- wait_until: { frames: 60 }\n",
        );
        // Refused before the frame that reads it: the file is read where the decide at the last screen
        // before a stage is, and the walk above has not reached one yet.
        let dir = game.launch().dir().to_path_buf();
        for (inside, _) in &files {
            if is_the_runs_file(inside, SLOT) {
                game.sim().files().refuses_to_read(dir.join(inside));
            }
        }
        played_out(&game);
        let log = game.log();

        assert!(
            log.said("resume: cannot read"),
            "a file that could not be read was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said(&format!("resume: nothing was left of {SLOT}")),
            "a read that failed did not answer as no file at all does:\n  {}",
            log.lines().join("\n  ")
        );
        // And it is still there: a file worth looking at is one orb leaves for somebody to look at.
        assert!(
            game.sim().files().paths().iter().any(|path| {
                path.strip_prefix(&dir)
                    .is_ok_and(|inside| is_the_runs_file(inside, SLOT))
            }),
            "the file orb could not read was taken away"
        );
    });
}

/// A file that is there and is not one of orb's: the bytes make no sense of themselves, and the answer is
/// the same as no file at all.
#[test]
fn a_file_of_th06nc_that_cannot_be_made_sense_of_is_named_and_left_where_it_is() {
    in_its_own_process(|| {
        let mut files = a_run_left_behind("th06nc-left-nonsense");
        let nonsense = b"this is not a run".to_vec();
        for (inside, bytes) in &mut files {
            if is_the_runs_file(inside, SLOT) {
                *bytes = nonsense.clone();
            }
        }
        let game = the_answer(
            "th06nc-nonsense",
            &files,
            0,
            "- wait_until: { frames: 60 }\n",
        );
        played_out(&game);
        let log = game.log();

        assert!(
            log.said("resume: cannot make sense of") && log.said("it is left alone"),
            "a file that made no sense was not named, or was not said to be left alone:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said(&format!("resume: nothing was left of {SLOT}")),
            "a file that made no sense did not answer as no file at all does:\n  {}",
            log.lines().join("\n  ")
        );
        // **The bytes are not asserted, and the file is.** What "left alone" is about is the *read*: orb
        // neither removed the file nor wrote over it to get past it. The run that then starts is the run
        // the game was going to start, and reaching its own first chapter writes its own file there — so
        // bytes held against the nonsense would be asserting that a run keeps nothing of itself.
        let dir = game.launch().dir().to_path_buf();
        assert!(
            game.sim().files().paths().iter().any(|path| {
                path.strip_prefix(&dir)
                    .is_ok_and(|inside| is_the_runs_file(inside, SLOT))
            }),
            "the file orb could not make sense of was taken away"
        );
    });
}

/// A file holding another run's chapter: the run about to be played is not the one on disk, so there is
/// nothing to offer and the file is another run's to keep.
#[test]
fn a_file_of_th06nc_holding_another_runs_chapter_is_named_and_left_alone() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-another-run");
        // The same bytes under the other shot's name, and this run's own file gone: what the launch finds
        // is a run of somebody else's.
        let elsewhere: Vec<(PathBuf, Vec<u8>)> = files
            .iter()
            .map(|(inside, bytes)| {
                if is_the_runs_file(inside, SLOT) {
                    let said = inside.to_string_lossy().replace(SLOT, ANOTHER_SHOTS_SLOT);
                    (PathBuf::from(said), bytes.clone())
                } else {
                    (inside.clone(), bytes.clone())
                }
            })
            .collect();
        let game = the_answer(
            "th06nc-another-run",
            &elsewhere,
            0,
            "- wait_until: { frames: 60 }\n",
        );
        played_out(&game);
        let log = game.log();

        // No question about a run this launch is not playing: the answer step gives up rather than being
        // taken, and what says so is the question never having been asked.
        assert!(
            !log.said(&format!("resume: {SLOT} was left")),
            "another run's file was offered as this run's:\n  {}",
            log.lines().join("\n  ")
        );
        let dir = game.launch().dir().to_path_buf();
        assert!(
            game.sim().files().paths().iter().any(|path| {
                path.strip_prefix(&dir)
                    .is_ok_and(|inside| is_the_runs_file(inside, ANOTHER_SHOTS_SLOT))
            }),
            "the other run's file was taken away by a launch that had no claim on it"
        );
    });
}

/// A write the host refuses: the run plays on, and what it cannot keep is said rather than raised.
#[test]
fn a_file_of_th06nc_that_cannot_be_written_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- wait_until: {{ frames: 120 }}
- quit: true
"
        );
        let game = launched("th06nc-unwritable", recipe);
        // Refused at the path the run's own file goes to, before the chapter that writes it.
        let at = game
            .launch()
            .dir()
            .join(KEPT_IN)
            .join(format!("{SLOT}.{KEPT_AS}"));
        game.sim().files().refuses_to_write(&at);
        played_out(&game);
        let log = game.log();

        assert!(
            log.said("resume: cannot write"),
            "a write the host refused was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And the run reached the chapter it was playing for, which is what "plays on" means.
        assert!(
            log.said("stage 1 chapter 1"),
            "a run whose file could not be written did not play on:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A directory the file cannot go in: the run plays on, and the chapter simply is not written down.
#[test]
fn a_directory_the_file_of_th06nc_cannot_go_in_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- wait_until: {{ frames: 120 }}
- quit: true
"
        );
        let game = launched("th06nc-no-directory", recipe);
        // A file standing where the directory has to go, which is the one thing that makes
        // `create_dir_all` refuse a path it would otherwise make.
        let at = game.launch().dir().join(KEPT_IN);
        game.sim()
            .files()
            .put(&at, "a file where the directory goes");
        played_out(&game);
        let log = game.log();

        assert!(
            log.said(&format!("resume: cannot make {}", at.display())),
            "the directory the file could not go in was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // It cost the run nothing but the chapter not being written down: the chapter itself happened and
        // the run went on.
        assert!(
            log.said("stage 1 chapter 1"),
            "a run whose file had nowhere to go did not play on:\n  {}",
            log.lines().join("\n  ")
        );
        // And nothing is left for a later launch to offer, the file the directory would have held being
        // the only thing that could be.
        let dir = game.launch().dir().to_path_buf();
        assert!(
            !game.sim().files().paths().iter().any(|path| {
                path.strip_prefix(&dir)
                    .is_ok_and(|inside| is_the_runs_file(inside, SLOT))
            }),
            "a chapter file was written where the directory could not be made"
        );
    });
}

/// A stage that is not the one the file asked for: nothing of the run goes in, which is the only safe
/// answer to a stage nobody asked for.
///
/// **What this protects is the one thing about a resume nothing else can catch.** The numbers orb holds
/// were read as *that* stage began, so writing them over another stage is a run put back wrong with
/// nothing about the file saying so — the run would play on with another stage's clock, generator and
/// lives.
#[test]
fn a_stage_of_th06nc_other_than_the_one_asked_for_is_not_written_over() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-another-stage");
        // つづきから, and then stopped before the stage the run was left in is built: what comes up
        // instead is the second stage, put up by this test rather than by the recipe.
        let game = the_answer(
            "th06nc-another-stage",
            &files,
            0,
            "- wait_until: { frames: 30 }\n",
        );
        let log = game.log();
        the_run_is_asked_for(&game);

        // The second stage where the first was asked for, counted from zero.
        game.builds_the_stage(1);
        for _ in 0..600 {
            if log.said("resume: not put back — stage 2 came up where stage 1 was asked for") {
                break;
            }
            game.frame();
        }
        assert!(
            log.said("resume: not put back — stage 2 came up where stage 1 was asked for"),
            "a stage nobody asked for was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And nothing of the run went in: no buttons were played into a stage the file is not about.
        assert!(
            !log.said("playing its buttons in"),
            "a stage nobody asked for had the run played into it:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A run orb keeps nothing of, which is the same protection from the other side: the stage the file asked
/// for does come up, and it comes up inside a demo.
#[test]
fn a_run_of_th06nc_orb_keeps_nothing_of_is_not_written_over_either() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-for-a-demo");
        let game = the_answer("th06nc-a-demo", &files, 0, "- wait_until: { frames: 30 }\n");
        let log = game.log();
        the_run_is_asked_for(&game);

        // The demo, which is what the replay list's own screens come to rest in, and then the stage a run
        // gets.
        game.image().a_replay_is_playing(true);
        game.builds_the_stage(0);
        for _ in 0..600 {
            if log.said("resume: not put back — what came up is a demo") {
                break;
            }
            game.frame();
        }
        assert!(
            log.said("resume: not put back — what came up is a demo"),
            "a run orb keeps nothing of was written over:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("playing its buttons in"),
            "a demo had the run played into it:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And a stage that never comes up at all: given up on rather than waited for.
#[test]
fn a_stage_of_th06nc_that_never_comes_up_is_given_up_on() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-for-no-stage");
        let game = the_answer(
            "th06nc-no-stage",
            &files,
            0,
            "- wait_until: { frames: 30 }\n",
        );
        let log = game.log();
        the_run_is_asked_for(&game);

        // No build at all, which is a stage the game cannot load.
        game.the_stage_never_comes_up();
        for _ in 0..(RESUME_WAIT + 120) {
            if log.said("resume: given up") {
                break;
            }
            game.frame();
        }
        assert!(
            log.said("resume: given up"),
            "a stage that never came up was waited for rather than given up on:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// How long orb holds a run's numbers for a stage that has been asked for, in frames.
///
/// Written out rather than taken from `resume`, where it is private: what this is about is a bound
/// somebody waiting at the screen would notice, and a test that borrowed orb's own number could not fail
/// if orb changed it.
const RESUME_WAIT: u32 = 600;

/// Frames run until the run left behind has been asked for, which is the moment every stage test above
/// starts from: つづきから answered, and the stage the file names asked of the game.
fn the_run_is_asked_for(game: &Fake) {
    let log = game.log();
    for _ in 0..FRAMES {
        if log.said("the run is being built at that stage") {
            return;
        }
        if game.sim().exited().is_some() {
            panic!(
                "the recipe quit before the run was asked for:\n  {}",
                log.lines().join("\n  ")
            );
        }
        game.frame();
    }
    panic!(
        "the run left behind was never asked for in {FRAMES} frames:\n  {}",
        log.lines().join("\n  ")
    );
}

/// A launch that finds `files` and is walked at the keyboard to the question, leaving the press that
/// answers it to the test.
///
/// **No recipe at all**, unlike every other launch in this file: a recipe's `Step::Answer` takes an item,
/// and the cancel is the answer that is no item of the question — so it is a key, and a key of a person's
/// is one orb only reads where nothing is being fed. The walk is therefore the game's own screens pressed
/// through, the way `the_mode_question_over_th06nc` presses its own question up: orb's question about the
/// mode on the first press at the menu, and then the three selects and the mode select, **which this
/// game's own front end hands on by itself** — see `fake::th06nc`, whose difficulty, character and shot
/// each go by after `A_SELECT_GOES_BY_ON_ITS_OWN_AFTER` frames, where th06c's want a decide apiece.
fn at_the_question_on_the_keyboard(name: &str, files: &[(PathBuf, Vec<u8>)]) -> Box<Fake> {
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.stages_last(A_STAGE);
    let log = game.log();
    // The canvas first, which is what a question of orb's is drawn through on this game — a press before
    // one is a press at a menu nothing is over. See `the_mode_question_over_th06nc`, which waits the same
    // out.
    game.frames_until("a canvas over this game's swapchain", FRAMES, || {
        log.said("canvas: drawing through this game's own Direct3D 11, over 640x480")
    });
    // **And the title before the menu**, which is the step th06c's walk has no need of: that game's title
    // *is* its menu, and this one's opens one on a decide — `TO_A_RUN` waits the same sixty frames out
    // before pressing, the title reading nothing over them.
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
    });
    game.frames(A_TITLES_OWN_FRAMES);
    // **One run of presses for both screens, and not one apiece**, which is the thing this got wrong
    // first: the taps that open the menu carry on through the twenty frames it ignores a press for, so
    // the tap that raises orb's question is one of *those* — a second loop started afterwards is looking
    // for a line already written, and waited out its thirty presses over a question that was up.
    let from = log.written();
    game.press_until(keys::Z, "orb's own question about the mode", || {
        log.said_since(from, "menu: Run is under the cursor, asking which mode")
    });
    game.frames(READS_KEYS_AFTER);
    game.press_until(keys::Z, "the mode chosen", || {
        log.said_since(from, "mode: answered on the")
    });
    // The press orb held back for that question is handed on to the menu, whose first item is what walks
    // to the difficulty select — see `the_mode_question_over_th06nc`, which says the same of it — and then
    // the three selects and the mode select, tapped through to the screen the question stands over.
    //
    // **Tapped every other frame, looked at every frame**, the selects waiting for a press as a launch of the
    // real game showed — it stood on screen 11 for 1,200 frames with nothing pressed at it — and the walk
    // stopping on the screen the question stands over before a tap reaches it.
    let from = log.written();
    let mut down = false;
    for _ in 0..FRAMES {
        if game.image().the_screen_the_front_end_is_on() == THE_LAST_SCREEN {
            break;
        }
        down = !down;
        game.keyboard().set(keys::Z, down);
        game.frame();
    }
    game.keyboard().set(keys::Z, false);
    game.frames_until("the last screen before a stage", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == THE_LAST_SCREEN
    });
    game.frames(READS_KEYS_AFTER);
    game.press_until(keys::Z, "the question about the run left behind", || {
        log.said_since(
            from,
            &format!("resume: {SLOT} was left; asking where to start"),
        )
    });
    game
}

/// The one screen before a stage that takes a press, which is where orb's question about picking a run up
/// stands — `front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS`.
///
/// Written out rather than taken from `Th06nc`, which keeps the same number: a test that read the screen
/// off the code it is about would agree with it whatever either said.
const THE_LAST_SCREEN: i32 = 24;

/// And a cancel: no run at all, the file untouched, and the screen the question went up over carrying on.
#[test]
fn cancelling_the_question_of_th06nc_starts_no_run_and_leaves_the_last_screen_alone() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th06nc-left-for-a-cancel");
        let game = at_the_question_on_the_keyboard("th06nc-cancelled", &files);
        let log = game.log();

        // **Held down rather than pressed and let go**, which is what the key that cancelled really is: it
        // was down when the question went up, and what it must not do is reach the screen underneath.
        //
        // **And the screen underneath would walk if the key reached it**, which is what makes this worth
        // asserting: a launch of `the-cancel-at-the-last-screen.yaml` tapped the cancel at 24 and read
        // `front end: screen 14`, then tapped again and read `13`. `fake::th06nc` carries those two arms
        // because of that launch — before it, the screen standing still below was true of the laid-out
        // game by omission.
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

        assert!(!game.state().playing, "a cancelled question started a run");
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            THE_LAST_SCREEN,
            "the key that cancelled took the front end off the screen under the question",
        );
        // And the file is still there: the press that would have started the run was never handed over, so
        // nothing of the run left behind was read back or written over.
        let files = left_by(&game, game.launch().dir());
        assert!(
            files
                .iter()
                .any(|(inside, _)| is_the_runs_file(inside, SLOT)),
            "the run left unfinished was disturbed by a question nobody answered: {:?}",
            files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
        );
    });
}
