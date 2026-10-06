//! **The cases the real th06c is tested with, played against the laid-out th06c first** — the 64-bit half of
//! `the_cases_the_real_game_is_tested_with`, which is that file's two tests for the two 32-bit games and
//! cannot be one file with them: a laid-out game is the architecture the real one is, and this crate is built
//! once per architecture with the modules its games' tests are in.
//!
//! What it says and why is that file's: a recipe whose step never reaches its point, or a check that asks for
//! something no launch shows, fails here before it costs a launch of the real game — and the laid-out game is
//! held to walking where the real one walked, by the traces `cargo xtask real` left where
//! `ORB_REAL_TRACES_TH06C` names them.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, Start, TRACE_FILE, Trace};

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in — see the 32-bit file's own.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing into the stage after it can.
const A_STAGE: u32 = 6_000;

/// What the launch before left under its directory, re-rooted under this one — the trace left out, for the
/// reason the 32-bit file gives.
fn left_by(game: &Fake, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            let inside = path.strip_prefix(dir).ok()?.to_path_buf();
            if inside == Path::new(TRACE_FILE) {
                return None;
            }
            Some((inside, files.get(&path)?))
        })
        .collect()
}

enum Held {
    NotCompared,
    Same,
    Different(String),
}

fn played_out(game: &Fake, dir: &Path, case: &Case, against: &[PathBuf]) -> Held {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            break;
        }
        game.frame();
    }
    if game.sim().exited().is_none() {
        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| !line.contains("resume: frame ") && !line.contains(" scene="))
            .collect();
        panic!(
            "{} {} never quit in {FRAMES} frames on the laid-out game; what it said:\n  {}",
            case.game.name(),
            case.name,
            said.join("\n  ")
        );
    }
    orb_core::trace::close();
    let trace = game
        .sim()
        .files()
        .get(dir.join(TRACE_FILE))
        .expect("the launch wrote a trace");
    let trace = Trace::read(&trace).expect("the trace reads back");
    if let Err(why) = (case.check)(&trace) {
        panic!(
            "{} {} failed on the laid-out game: {why}\n  what the launch said:\n    {}",
            case.game.name(),
            case.name,
            trace.narrative().replace('\n', "\n    "),
        );
    }
    held_against_the_real_game(case, &trace, against)
}

fn real_traces(game: Game) -> Vec<PathBuf> {
    std::env::var(format!("ORB_REAL_TRACES_{}", game.name().to_uppercase()))
        .unwrap_or_default()
        .split(';')
        .filter(|named| !named.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn held_against_the_real_game(case: &Case, laid_out: &Trace, sessions: &[PathBuf]) -> Held {
    let Some(bytes) = sessions
        .iter()
        .find_map(|session| std::fs::read(session.join(case.name).join(TRACE_FILE)).ok())
    else {
        return Held::NotCompared;
    };
    let real = Trace::read(&bytes).expect("the real game's trace reads back");
    let (ours, theirs) = (laid_out.shape(), real.shape());
    let Some(at) = (0..ours.len().max(theirs.len())).find(|at| ours.get(*at) != theirs.get(*at))
    else {
        return Held::Same;
    };
    let said = |shape: &[String]| {
        shape
            .iter()
            .enumerate()
            .map(|(step, line)| format!("      {}{line}", if step == at { "> " } else { "  " }))
            .collect::<Vec<_>>()
            .join("\n")
    };
    Held::Different(format!(
        "{} {}: parted at {at}, the laid-out game walking {} places and the real one {}\n    \
         the laid-out game:\n{}\n    the real game:\n{}",
        case.game.name(),
        case.name,
        ours.len(),
        theirs.len(),
        said(&ours),
        said(&theirs),
    ))
}

fn settings(case: &Case) -> impl FnOnce(&mut orb_config::Config) {
    let recipe = case.recipe.to_owned();
    move |config| {
        config.autoplay = Some(recipe);
        config.trace = Some(PathBuf::from(TRACE_FILE));
        config.log_level = LogLevel::Verbose;
        // **Pressed through the virtual pad, as `cargo xtask real` launches every case.**
        config.through_a_pad = true;
    }
}

/// **Every case of th06c's, each the test of its own below** — held against `orb-real`'s own list by
/// [`every_case_of_th06c_has_a_test_of_its_own`], so a case added there fails this file rather than going
/// unrun. **A test per case**, so that they are played at once rather than one after another: one test for
/// all sixteen once took 181s of a 181s run, eleven tests on eight threads.
const CASES: [&str; 20] = [
    "a_run_left_in_its_second_chapter",
    "the_run_picked_up_where_it_was_left",
    "a_death_answered_by_the_chapter_again",
    "the_run_started_over_and_its_replay_kept",
    "an_attempt_of_the_kept_replay_watched",
    "a_chapter_taken_again_without_a_death",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_of_the_stage_after_the_first",
    "the_run_picked_up_in_the_stage_it_walked_into",
    "a_stage_practised_on_its_own",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_run_played_to_its_clear",
    "a_normal_mode_run_played_to_its_clear",
    "the_extra_a_cleared_run_unlocked",
    "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
];

/// **Every case `orb-real` has is named in [`CASES`], and every name there is a case** — which is what makes
/// a case added there reach a test here.
#[test]
fn every_case_of_th06c_has_a_test_of_its_own() {
    let names: Vec<&str> = orb_real::cases_of(Game::Th06c)
        .iter()
        .map(|case| case.name)
        .collect();
    assert_eq!(
        names, CASES,
        "th06c's cases are not the ones this file names"
    );
}

/// **One case played out on the laid-out game as `xtask real` plays it alone**: its records holding a clear
/// only where it starts from one, `Case::starts_from`, then each case it plays first and itself last, each
/// launch finding what the one before left.
fn the_case_passes(name: &str) {
    in_its_own_process(|| {
        let against = real_traces(Game::Th06c);
        let case = orb_real::case_of(Game::Th06c, name)
            .unwrap_or_else(|| panic!("th06c has no case called {name}"));
        let a_clear = case.starts_from != Start::Nothing;
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        let launches = case.before.iter().map(|before| {
            orb_real::case_of(Game::Th06c, before).unwrap_or_else(|| {
                panic!("{name} plays {before} first, which is no case of th06c's")
            })
        });
        let mut parted = None;
        let mut held = false;
        for launch in launches.chain(std::iter::once(case)) {
            let game = Fake::attach_configured(
                Display::agreed(120),
                &format!("th06c-real-{name}-{}", launch.name),
                Work::wandering(WORK_US),
                settings(&launch),
            );
            game.stages_last(A_STAGE);
            if !a_clear {
                game.its_records_hold_no_clear();
            }
            let dir = game.launch().dir().to_path_buf();
            for (inside, bytes) in &left {
                game.sim().files().put(dir.join(inside), bytes);
            }
            match played_out(&game, &dir, &launch, &against) {
                Held::NotCompared => {}
                Held::Same => held = true,
                Held::Different(how) => {
                    held = true;
                    parted = Some(how);
                }
            }
            left = left_by(&game, &dir);
        }
        if let Some(how) = parted {
            panic!("the laid-out game did not walk where the real one did:\n  {how}");
        }
        assert!(
            against.is_empty() || held,
            "{against:?} holds no trace of {name}",
        );
    });
}

/// One `#[test]` per case.
macro_rules! a_test_per_case {
    ($($test:ident => $name:literal,)*) => {
        $(
            #[test]
            fn $test() {
                the_case_passes($name);
            }
        )*
    };
}

a_test_per_case!(
    a_run_of_th06c_left_in_its_second_chapter => "a_run_left_in_its_second_chapter",
    the_run_of_th06c_picked_up_where_it_was_left => "the_run_picked_up_where_it_was_left",
    a_death_of_th06c_answered_by_the_chapter_again => "a_death_answered_by_the_chapter_again",
    the_run_of_th06c_started_over_and_its_replay_kept => "the_run_started_over_and_its_replay_kept",
    an_attempt_of_th06cs_kept_replay_watched => "an_attempt_of_the_kept_replay_watched",
    a_chapter_of_th06c_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    a_chapter_of_th06c_further_back_taken_again => "a_chapter_further_back_taken_again",
    a_death_of_th06c_answered_by_a_chapter_further_back => "a_death_answered_by_a_chapter_further_back",
    a_death_in_the_nonspell_after_a_card_of_th06c_answered_by_that_nonspell => "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    the_stage_of_th06c_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th06c_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th06cs_stage_after_the_first => "a_chapter_of_the_stage_after_the_first",
    the_run_of_th06c_picked_up_in_the_stage_it_walked_into => "the_run_picked_up_in_the_stage_it_walked_into",
    a_stage_of_th06c_practised_on_its_own => "a_stage_practised_on_its_own",
    a_normal_mode_run_of_th06c_nothing_is_kept_of => "a_normal_mode_run_nothing_is_kept_of",
    a_run_of_th06c_played_to_its_clear => "a_run_played_to_its_clear",
    a_normal_mode_run_of_th06c_played_to_its_clear => "a_normal_mode_run_played_to_its_clear",
    the_extra_of_th06c_a_cleared_run_unlocked => "the_extra_a_cleared_run_unlocked",
    the_extra_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb => "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    a_normal_run_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb => "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
);
