//! **The answers to どこから始める over a run of th10 that do not put one back, and the ways the file behind
//! it is declined.**
//!
//! `a_pointdevice_run_of_th10.rs` answers つづきから, which is the one that does. The other answer and the
//! ways the file can be wrong are here, for the reason `the_run_of_th08_left_behind.rs` gives: a launch that
//! read a file it should have declined puts a run back wrong, and one that stops over a file it could not read
//! takes a run away from somebody who still has it on disk.
//!
//! **はじめから** starts the run the game was going to start and leaves the file for that run's own first
//! chapter to write over. A file that cannot be read, cannot be made sense of, or holds another run's chapter
//! is named and **left where it is**; a file that cannot be written, or a directory it cannot go in, is named
//! and the run plays on. A stage other than the one asked for, one inside the attract demo, and one that never
//! comes up are each declined rather than written over. **And a cancel, which over this game is no answer**, as
//! over th08: the question goes up on the frame the shot select asked for the run, with nothing left to
//! withhold.
//!
//! **Every answer is a recipe's but the cancel's**; the cancel is no item of the question, so it is pressed at
//! the keyboard over a launch with no recipe in it.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_core::game::th10::image::{STEP_READING, Screen};
use orb_sim::keys;

use crate::fake::th10::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long the laid-out stage lasts, which is past its midboss's arrival at frame 1200: long enough for the
/// stage's second chapter, which is what these leave a run in.
const A_STAGE: u32 = 4_500;

/// Where orb keeps the runs left unfinished, and what it calls one.
///
/// Written out here rather than taken from `resume`, where both are private: what an e2e test says it left
/// behind is a file at a path, and one that borrowed orb's own idea of that path could not fail if orb changed
/// it.
const KEPT_IN: &str = "pointdevice_resume";
const KEPT_AS: &str = "msgpack";

/// The name orb gives the run these play, which names its file: Normal, and Reimu with her first shot.
///
/// Written out rather than asked of `run_slot`, which would be this agreeing with the code it is about.
const SLOT: &str = "normal-reimua";

/// And the same difficulty with Reimu's second shot, which is a run of its own with a file of its own.
const ANOTHER_SHOTS_SLOT: &str = "normal-reimub";

/// How long orb holds a run's numbers for a stage that has been asked for, in frames.
///
/// Written out rather than taken from `resume`, where it is private: what this is about is a bound somebody
/// waiting at the screen would notice, and a test that borrowed orb's own number could not fail if orb changed
/// it.
const RESUME_WAIT: u32 = 600;

fn configured(config: &mut orb_config::Config, recipe: Option<String>) {
    config.autoplay = recipe;
    config.log_level = LogLevel::Verbose;
    // Every replay kept without asking, these being about the chapter file: a run given up leaves a replay of
    // itself too, and left to be asked that question would stand where this one wants to.
    config.save_replays = true;
}

/// The laid-out stage these play: long enough for a second chapter, and firing nothing, a run's dying being
/// none of this file's subject.
fn over_its_stage(game: &Fake) {
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game.fires_at_the_player(false);
}

/// A launch of this game played by `recipe`, over a stage of its own.
fn launched(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| configured(config, Some(recipe)),
    );
    over_its_stage(&game);
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

/// Whether a path under orb's directory is the file a run of `slot` is left in.
fn is_the_runs_file(inside: &Path, slot: &str) -> bool {
    let said = inside.to_string_lossy();
    said.contains(KEPT_IN) && said.contains(slot) && said.ends_with(KEPT_AS)
}

/// Whether the launch's directory still holds the file of `slot`.
fn still_holds(game: &Fake, slot: &str) -> bool {
    let dir = game.launch().dir().to_path_buf();
    game.sim().files().paths().iter().any(|path| {
        path.strip_prefix(&dir)
            .is_ok_and(|inside| is_the_runs_file(inside, slot))
    })
}

/// A run played to its stage's second chapter and left there, and the files that launch left behind.
///
/// The second chapter and not the first: a chapter file is written as a chapter is *reached*, so a run left in
/// the first has been through one write and a run left in the second through two, which is what says the file
/// being read back is the run's own and not its opening.
fn a_run_left_behind(name: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
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

/// A launch that finds `files` and walks to the question, answering it with `item`, then plays on for as long
/// as `after` says and quits.
fn the_answer(name: &str, files: &[(PathBuf, Vec<u8>)], item: usize, after: &str) -> Box<Fake> {
    let recipe = format!(
        "{TO_THE_SHOT}\
- press: [decide]
  frames: 10
  until: {{ question: resume }}
  give_up_after: 900
- answer: {{ question: resume, item: {item} }}
  give_up_after: 600
{after}\
- quit: true
"
    );
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| configured(config, Some(recipe)),
    );
    over_its_stage(&game);
    game
}

/// はじめから: the run the game was going to start, with the file left standing until this run's own first
/// chapter writes over it.
#[test]
fn starting_a_run_of_th10_from_the_beginning_leaves_the_file_for_its_own_first_chapter() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-for-the-beginning");
        let game = the_answer(
            "th10-from-the-beginning",
            &files,
            1,
            "\
- play_until: { stage: 1, chapter: 1 }
  give_up_after: 1800
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
        // The run is the game's own: nothing of the file was played into it, which is what the landing being
        // unsaid means.
        assert!(
            !log.said("resume: the landing is"),
            "a run started from the beginning had the file played into it:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("stage 1 chapter 1"),
            "the run did not begin at its own first chapter:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A file that cannot be read at all — a read the host refuses, which is how an e2e test says the read failed
/// without inventing a file that cannot exist.
#[test]
fn a_file_of_th10_that_cannot_be_read_is_named_and_costs_the_run_nothing() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-unreadable");
        let game = the_answer(
            "th10-unreadable",
            &files,
            0,
            "- wait_until: { frames: 60 }\n",
        );
        // Refused before the frame that reads it: the file is read where the shot select asks for the run, and
        // the walk has not reached one yet.
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
        assert!(
            still_holds(&game, SLOT),
            "the file orb could not read was taken away"
        );
    });
}

/// A file that is there and is not one of orb's: the bytes make no sense of themselves, and the answer is the
/// same as no file at all.
#[test]
fn a_file_of_th10_that_cannot_be_made_sense_of_is_named_and_left_where_it_is() {
    in_its_own_process(|| {
        let mut files = a_run_left_behind("th10-left-nonsense");
        for (inside, bytes) in &mut files {
            if is_the_runs_file(inside, SLOT) {
                *bytes = b"this is not a run".to_vec();
            }
        }
        let game = the_answer("th10-nonsense", &files, 0, "- wait_until: { frames: 60 }\n");
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
        // **The bytes are not asserted, and the file is**, for `the_run_of_th06c_left_behind`'s reason: the run
        // that then starts writes its own file there as it reaches its own first chapter.
        assert!(
            still_holds(&game, SLOT),
            "the file orb could not make sense of was taken away"
        );
    });
}

/// A file holding another run's chapter: the run about to be played is not the one on disk, so there is
/// nothing to offer and the file is another run's to keep.
#[test]
fn a_file_of_th10_holding_another_runs_chapter_is_named_and_left_alone() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-another-run");
        // The same bytes under the other shot's name, and this run's own file gone: what the launch finds is a
        // run of somebody else's.
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
            "th10-another-run",
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
        assert!(
            still_holds(&game, ANOTHER_SHOTS_SLOT),
            "the other run's file was taken away by a launch that had no claim on it"
        );
    });
}

/// A run played to its second chapter and quit, with `refused` done to its directory before it starts.
fn a_run_whose_file_is_refused(name: &str, refused: impl FnOnce(&Fake)) -> Box<Fake> {
    let recipe = format!(
        "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- wait_until: {{ frames: 120 }}
- quit: true
"
    );
    let game = launched(name, recipe);
    refused(&game);
    played_out(&game);
    game
}

/// A write the host refuses: the run plays on, and what it cannot keep is said rather than raised.
#[test]
fn a_file_of_th10_that_cannot_be_written_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let game = a_run_whose_file_is_refused("th10-unwritable", |game| {
            let at = game
                .launch()
                .dir()
                .join(KEPT_IN)
                .join(format!("{SLOT}.{KEPT_AS}"));
            game.sim().files().refuses_to_write(&at);
        });
        let log = game.log();

        assert!(
            log.said("resume: cannot write"),
            "a write the host refused was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("stage 1 chapter 2"),
            "a run whose file could not be written did not play on:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A directory the file cannot go in: the run plays on, and the chapter simply is not written down.
#[test]
fn a_directory_the_file_of_th10_cannot_go_in_is_named_and_the_run_plays_on() {
    in_its_own_process(|| {
        let mut at = PathBuf::new();
        let game = a_run_whose_file_is_refused("th10-no-directory", |game| {
            // A file standing where the directory has to go, which is the one thing that makes
            // `create_dir_all` refuse a path it would otherwise make.
            at = game.launch().dir().join(KEPT_IN);
            game.sim()
                .files()
                .put(&at, "a file where the directory goes");
        });
        let log = game.log();

        assert!(
            log.said(&format!("resume: cannot make {}", at.display())),
            "the directory the file could not go in was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("stage 1 chapter 2"),
            "a run whose file had nowhere to go did not play on:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !still_holds(&game, SLOT),
            "a chapter file was written where the directory could not be made"
        );
    });
}

/// Frames run until the run left behind has been asked for, which is the moment every stage test below starts
/// from: つづきから answered, and the stage the file names asked of the game.
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

/// Frames until `said`, or the bound, and whether it was said.
fn frames_until_said(game: &Fake, said: &str, bound: u32) -> bool {
    let log = game.log();
    for _ in 0..bound {
        if log.said(said) {
            return true;
        }
        game.frame();
    }
    log.said(said)
}

/// A stage that is not the one the file asked for: nothing of the run goes in, which is the only safe answer
/// to a stage nobody asked for — the numbers orb holds were read as *that* stage began.
#[test]
fn a_stage_of_th10_other_than_the_one_asked_for_is_not_written_over() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-another-stage");
        let game = the_answer(
            "th10-another-stage",
            &files,
            0,
            "- wait_until: { frames: 30 }\n",
        );
        let log = game.log();
        the_run_is_asked_for(&game);

        // The second stage where the first was asked for, counted from zero.
        game.builds_the_stage_instead(1);
        let said = "resume: not put back — stage 2 came up where stage 1 was asked for";
        assert!(
            frames_until_said(&game, said, 600),
            "a stage nobody asked for was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("playing its buttons in"),
            "a stage nobody asked for had the run played into it:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A run orb keeps nothing of, which is the same protection from the other side: the stage the file asked for
/// does come up, and it comes up inside the attract demo.
#[test]
fn a_run_of_th10_orb_keeps_nothing_of_is_not_written_over_either() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-for-a-demo");
        let game = the_answer("th10-a-demo", &files, 0, "- wait_until: { frames: 30 }\n");
        let log = game.log();
        the_run_is_asked_for(&game);

        game.builds_it_inside_the_demo();
        let said = "resume: not put back — what came up is a demo";
        assert!(
            frames_until_said(&game, said, 600),
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
fn a_stage_of_th10_that_never_comes_up_is_given_up_on() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-for-no-stage");
        let game = the_answer("th10-no-stage", &files, 0, "- wait_until: { frames: 30 }\n");
        let log = game.log();
        the_run_is_asked_for(&game);

        game.the_stage_never_comes_up();
        assert!(
            frames_until_said(&game, "resume: given up", RESUME_WAIT + 120),
            "a stage that never came up was waited for rather than given up on:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// Whether the title is on `screen` and reading a key.
fn reading(game: &Fake, screen: Screen) -> bool {
    let (on, step, _, _) = game.image().where_the_title_is();
    game.image().the_title_is_up() && on == screen as i32 && step == STEP_READING
}

/// Waits for the title to be on `screen` and reading, and decides on it.
fn decides_on(game: &Fake, screen: Screen, until: impl Fn() -> bool) {
    game.frames_until(&format!("{screen:?}"), FRAMES, || reading(game, screen));
    for _ in 0..PRESSES {
        if until() {
            return;
        }
        game.press(keys::Z);
    }
    panic!(
        "{screen:?} was not decided in {PRESSES} presses:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// How many presses a screen is given to answer a decide.
const PRESSES: u32 = 30;

/// A launch that finds `files` and is walked at the keyboard to the question, leaving the press that answers
/// it to the test.
///
/// **No recipe at all**, unlike every other launch in this file: the cancel is the answer that is no item of
/// the question, so it is a key, and a key of a person's is one orb only reads where nothing is being fed. The
/// walk is therefore the game's own screens pressed through: the screen that waits for a key, orb's question
/// about the mode on the first decide at the main menu, then the difficulty, the character and the shot, each
/// of which comes up on the item this run is — Normal, Reimu and her first shot — so every one of them is a
/// decide and nothing else.
fn at_the_question_on_the_keyboard(name: &str, files: &[(PathBuf, Vec<u8>)]) -> Box<Fake> {
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| configured(config, None),
    );
    over_its_stage(&game);
    let log = game.log();
    game.frames_until("an overlay", FRAMES, || log.said("overlay: ready"));
    decides_on(&game, Screen::PressAKey, || {
        !reading(&game, Screen::PressAKey)
    });
    let from = log.written();
    decides_on(&game, Screen::MainMenu, || {
        log.said_since(from, "menu: Run is under the cursor, asking which mode")
    });
    game.frames(READS_KEYS_AFTER);
    game.press_until(keys::Z, "the mode chosen", || {
        log.said_since(from, "mode: answered on the")
    });
    for screen in [Screen::Difficulty, Screen::Character] {
        decides_on(&game, screen, || !reading(&game, screen));
    }
    let from = log.written();
    decides_on(&game, Screen::Shot, || {
        log.said_since(
            from,
            &format!("resume: {SLOT} was left; asking where to start"),
        )
    });
    game
}

/// **And a cancel is no answer to it over this game**, as over th08. The question goes up on the frame the shot
/// select asked for the run: its decide is no press orb holds back — `Th10::menu_takes_a_press` answers for the
/// main menu alone — and by that frame the title is on its way out, so there is no press to withhold and no
/// screen to carry on. So the question stays up through a cancel held as long as anybody likes, and the run
/// waits for one of its two items — see `resume_ui::Cancels::Nothing`.
#[test]
fn a_cancel_at_the_question_of_th10_is_no_answer_and_the_run_waits_for_one() {
    in_its_own_process(|| {
        let files = a_run_left_behind("th10-left-for-a-cancel");
        let game = at_the_question_on_the_keyboard("th10-cancelled", &files);
        let log = game.log();

        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.keyboard().set(keys::X, true);
        game.frames(60);
        game.keyboard().set(keys::X, false);
        game.frames(READS_KEYS_AFTER);
        assert!(
            !log.said_since(from, "resume: neither") && !log.said_since(from, "answered on the"),
            "a cancel answered the question:\n  {}",
            log.lines()[from..].join("\n  ")
        );
        assert!(
            !game.state().playing,
            "the run went on with the question unanswered"
        );
        assert!(
            still_holds(&game, SLOT),
            "the run left unfinished was disturbed by a question nobody answered"
        );

        // And one of its items answers it, the first being where its cursor starts.
        game.press_until(keys::Z, "the question answered", || {
            log.said_since(
                from,
                "resume: from where it stopped, answered on the keyboard",
            )
        });
    });
}

/// **The stage word of th10's last stage**, after which the panel asks for the ending (0x416eef).
const THE_LAST_STAGE: i32 = 6;

/// **A run cleared takes its file away, its ending skipped or not**: a Pointdevice run with a chapter file of its
/// own, its stage made the last so that winning it goes to the ending, which orb runs out inside one frame of its
/// own — and the next launch is then offered no run to pick up. Watched on the real game: a whole Normal cleared
/// with its ending skipped (`ending skipped, 13848 frames run, scene 14 -> 4`) left the file its last chapter wrote,
/// and the next launch's recipe stood at the question it put up.
#[test]
fn a_run_of_th10_cleared_with_its_ending_skipped_takes_its_file_away() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ frames: 20000 }}
  give_up_after: 24000
- quit: true
"
        );
        let game = launched("th10-cleared-with-its-ending-skipped", recipe);
        let log = game.log();
        game.frames_until("the run's second chapter", FRAMES, || {
            log.said("stage 1 chapter 2")
        });
        assert!(
            still_holds(&game, SLOT),
            "the run wrote no file of its own to take away:\n  {}",
            log.lines().join("\n  ")
        );
        game.image().the_stage_word_is(THE_LAST_STAGE);
        game.frames_until("the ending skipped", FRAMES, || log.said("ending skipped"));
        game.frames(120);
        assert!(
            !still_holds(&game, SLOT),
            "the file of a run cleared was left to be offered:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
