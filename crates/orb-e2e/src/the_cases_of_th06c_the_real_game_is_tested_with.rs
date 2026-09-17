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
use orb_real::{Case, Game, TRACE_FILE, Trace};

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
    }
}

/// **The case each chain starts at**, which is every case of th06c that continues nothing — see
/// `Case::continues`, and [`the_chain_from`], which is what one of these names.
///
/// **Written here and held against `orb-real`'s own list by
/// [`every_chain_of_th06cs_cases_has_a_test_of_its_own`]**, so a case added there with nothing before it
/// fails this file rather than going unrun.
///
/// **A test per chain and not one test for all sixteen**, which is the whole reason this list exists: the
/// cases were one test, and a chain has to be played in its own order — a case that continues is handed the
/// files the one before it left — so all sixteen ran one after another while the other seven threads of the
/// run sat idle. Measured: 181s of a 181s run was this one test, eleven tests on eight threads. Split, the
/// wall clock is the longest chain instead, which is the Normal clear and the Extra that continues it.
///
/// Eleven and not sixteen, the five chains longer than one case holding seven of them between them.
const CHAINS: [&str; 11] = [
    "a_run_left_in_its_second_chapter",
    "a_death_answered_by_the_chapter_again",
    "a_chapter_taken_again_without_a_death",
    "a_chapter_further_back_taken_again",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_of_the_stage_after_the_first",
    "a_stage_practised_on_its_own",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_run_played_to_its_clear",
    "a_normal_mode_run_played_to_its_clear",
];

/// The cases of the chain that starts at `head` — see `orb_real::chains_of`, which is what makes one.
///
/// # Panics
/// If no chain starts there, which is a chain this file names and `orb-real` does not have.
fn the_chain_from(head: &str) -> Vec<Case> {
    orb_real::chains_of(Game::Th06c)
        .into_iter()
        .find(|chain| chain[0].name == head)
        .unwrap_or_else(|| panic!("th06c has no chain starting at {head}"))
}

/// **Every chain [`CHAINS`] names is a chain `orb-real` has, and every chain it has is named** — which is
/// what makes a case added there reach a test here.
#[test]
fn every_chain_of_th06cs_cases_has_a_test_of_its_own() {
    let heads: Vec<&str> = orb_real::chains_of(Game::Th06c)
        .iter()
        .map(|chain| chain[0].name)
        .collect();
    assert_eq!(
        heads, CHAINS,
        "the chains this file plays are not the chains th06c's cases make",
    );
}

/// One chain played out on the laid-out game, in its own order and its own directory.
fn the_chain_passes(head: &str) {
    in_its_own_process(|| {
        let against = real_traces(Game::Th06c);
        let mut held = 0;
        let mut parted: Vec<String> = Vec::new();
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        let chain = the_chain_from(head);
        let names: Vec<&str> = chain.iter().map(|case| case.name).collect();
        for case in chain {
            let name = format!("th06c-real-{}", case.name);
            let game = Fake::attach_configured(
                Display::agreed(120),
                &name,
                Work::wandering(WORK_US),
                settings(&case),
            );
            game.stages_last(A_STAGE);
            if !case.after_a_clear() {
                game.its_records_hold_no_clear();
            }
            let dir = game.launch().dir().to_path_buf();
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
            }
            match played_out(&game, &dir, &case, &against) {
                Held::NotCompared => {}
                Held::Same => held += 1,
                Held::Different(how) => {
                    held += 1;
                    parted.push(how);
                }
            }
            left = left_by(&game, &dir);
        }
        assert!(
            parted.is_empty(),
            "the laid-out game did not walk where the real one did:\n  {}",
            parted.join("\n  ")
        );
        assert!(
            against.is_empty() || held > 0,
            "{against:?} holds no trace of any of {names:?}",
        );
    });
}

// One test per chain, so that the twelve of them are played at once rather than one after another — see
// [`CHAINS`], and [`every_chain_of_th06cs_cases_has_a_test_of_its_own`], which is what says none is
// missing. Named after the case each starts at, so a failure names the chain in the harness's own output.

#[test]
fn a_run_of_th06c_left_in_its_second_chapter_and_picked_up() {
    the_chain_passes("a_run_left_in_its_second_chapter");
}

#[test]
fn a_death_of_th06c_answered_by_the_chapter_and_the_replay_kept() {
    the_chain_passes("a_death_answered_by_the_chapter_again");
}

#[test]
fn a_chapter_of_th06c_taken_again_without_a_death() {
    the_chain_passes("a_chapter_taken_again_without_a_death");
}

#[test]
fn a_chapter_of_th06c_further_back_taken_again() {
    the_chain_passes("a_chapter_further_back_taken_again");
}

#[test]
fn the_stage_of_th06c_taken_again_from_its_start() {
    the_chain_passes("the_stage_taken_again_from_its_start");
}

#[test]
fn the_run_of_th06c_given_up_at_the_retry() {
    the_chain_passes("the_run_given_up_at_the_retry");
}

#[test]
fn a_chapter_of_th06cs_stage_after_the_first_and_the_run_picked_up_in_it() {
    the_chain_passes("a_chapter_of_the_stage_after_the_first");
}

#[test]
fn a_stage_of_th06c_practised_on_its_own() {
    the_chain_passes("a_stage_practised_on_its_own");
}

#[test]
fn a_normal_mode_run_of_th06c_nothing_is_kept_of() {
    the_chain_passes("a_normal_mode_run_nothing_is_kept_of");
}

#[test]
fn a_run_of_th06c_played_to_its_clear() {
    the_chain_passes("a_run_played_to_its_clear");
}

/// And the Extra that clear unlocked, which continues it in the directory its records were left in — so the
/// two are one chain and the longest of the eleven.
#[test]
fn a_normal_mode_run_of_th06c_played_to_its_clear_and_the_extra_it_unlocked() {
    the_chain_passes("a_normal_mode_run_played_to_its_clear");
}
