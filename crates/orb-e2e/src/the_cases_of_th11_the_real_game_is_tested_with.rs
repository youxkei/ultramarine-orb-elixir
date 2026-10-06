//! **The cases the real th11 is tested with, played against the laid-out th11 first** — the same thing
//! `the_cases_of_th10_the_real_game_is_tested_with` is for that game, and the reason is the same: a recipe whose
//! step never reaches its point, or a check that asks for something no launch shows, fails here in seconds instead
//! of costing a launch of the real game.
//!
//! **Every case is gated here, each by a test of its own**, which [`CASES`] names and
//! [`every_case_of_th11_has_a_test_of_its_own`] holds to `orb-real`'s own list — a case depending on no other, so
//! each is played the way `xtask real` plays it alone: its own launches first, `Case::before`, from the records it
//! starts from, `Case::starts_from`.
//!
//! **What the laid-out game can say about them and what it cannot** — th10's: it can say the recipe runs out, the
//! title walked by this game's own screen numbers, the run entered and played, the stages walked one after another
//! and the launch quit; it cannot say the addresses are this exe's, nor how far a run of the real game the rules
//! get.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, TRACE_FILE, Trace};

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in: six stages of [`A_STAGE`] and the recipes' own
/// waits either side of them.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing a whole run walks through stages rather than
/// sitting in the first — th10's, the fake's fights being th10's fake's.
const A_STAGE: u32 = 4_500;

/// **Every case of th11's, each the test of its own below** — see [`every_case_of_th11_has_a_test_of_its_own`].
const CASES: [&str; 24] = [
    "a_run_started_and_given_up",
    "a_normal_mode_run_played_without_a_bomb",
    "a_stage_practised_on_its_own",
    "a_death_answered_by_the_chapter_again",
    "the_run_started_over_and_its_replay_kept",
    "an_attempt_of_the_kept_replay_watched",
    "a_chapter_taken_again_without_a_death",
    "a_run_left_in_its_second_chapter",
    "the_run_picked_up_where_it_was_left",
    "a_chapter_of_the_stage_after_the_first",
    "the_run_picked_up_in_the_stage_it_walked_into",
    "a_normal_mode_run_nothing_is_kept_of",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "a_run_played_to_its_clear_without_a_bomb",
    "a_normal_mode_run_played_to_its_clear",
    "the_extra_played_to_its_clear",
    "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    "a_chapter_of_the_extra_taken_again",
    "an_attempt_at_a_card_counted_and_kept",
    "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
];

fn settings(case: &Case) -> impl FnOnce(&mut orb_config::Config) {
    let recipe = case.recipe.to_owned();
    move |config| {
        config.autoplay = Some(recipe);
        config.trace = Some(PathBuf::from(TRACE_FILE));
        config.log_level = LogLevel::Verbose;
        // **Pressed through the virtual pad, as `cargo xtask real` launches every case** — so that a recipe whose
        // press a pad cannot make fails here and not on the real game.
        config.through_a_pad = true;
    }
}

fn played_out(game: &Fake, dir: &Path, case: &Case) {
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
            .filter(|line| !line.contains(" scene=") && !line.contains("resume: frame "))
            .collect();
        panic!(
            "{} {} never quit in {FRAMES} frames on the laid-out game; the last of what it said:\n  {}",
            case.game.name(),
            case.name,
            said[said.len().saturating_sub(120)..].join("\n  ")
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
}

/// **Every case `orb-real` has is named in [`CASES`], and every name there is a case** — which is what makes a case
/// added there reach a test here.
#[test]
fn every_case_of_th11_has_a_test_of_its_own() {
    let names: Vec<&str> = orb_real::cases_of(Game::Th11)
        .iter()
        .map(|case| case.name)
        .collect();
    assert_eq!(
        names, CASES,
        "th11's cases are not the ones this file names"
    );
}

/// What the launch before left under its directory, re-rooted under this one — the trace left out, which is each
/// launch's own.
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

/// **And what it left beside itself**: the files opened by a bare name — `scoreth11.dat` and orb's fork of it —
/// which the case's next launch finds, the copy being one directory across a case's launches.
fn left_beside(game: &Fake) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter(|path| path.components().count() == 1)
        .filter_map(|path| Some((path.clone(), files.get(&path)?)))
        .collect()
}

/// **One case played out on the laid-out game as `xtask real` plays it alone**: the records it starts from laid
/// beside the game, made by `orb-real` with the game's own logic, then each case it plays first and itself last,
/// each launch finding what the one before left in orb's directory and beside the game.
fn the_case_passes(name: &str) {
    in_its_own_process(|| {
        let case = orb_real::case_of(Game::Th11, name)
            .unwrap_or_else(|| panic!("th11 has no case called {name}"));
        let mut beside: Vec<(PathBuf, Vec<u8>)> = Game::Th11
            .the_records_of(case.starts_from)
            .into_iter()
            .map(|(name, bytes)| (PathBuf::from(name), bytes))
            .collect();
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        let launches = case.before.iter().map(|before| {
            orb_real::case_of(Game::Th11, before).unwrap_or_else(|| {
                panic!("{name} plays {before} first, which is no case of th11's")
            })
        });
        for launch in launches.chain(std::iter::once(case)) {
            let game = Fake::attach_configured(
                Display::agreed(120),
                &format!("th11-real-{name}-{}", launch.name),
                Work::wandering(WORK_US),
                settings(&launch),
            );
            let dir = game.launch().dir().to_path_buf();
            for (inside, bytes) in &left {
                game.sim().files().put(dir.join(inside), bytes);
            }
            for (name, bytes) in &beside {
                game.sim().files().put(name, bytes);
            }
            // **The rules given a stage to play in**, th10's reason.
            game.plays_its_own_stage();
            game.stages_last(A_STAGE);
            played_out(&game, &dir, &launch);
            left = left_by(&game, &dir);
            beside = left_beside(&game);
        }
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
    a_run_of_th11_started_and_given_up => "a_run_started_and_given_up",
    a_normal_mode_run_of_th11_played_without_a_bomb => "a_normal_mode_run_played_without_a_bomb",
    a_stage_of_th11_practised_on_its_own => "a_stage_practised_on_its_own",
    a_death_of_th11_answered_by_the_chapter_again => "a_death_answered_by_the_chapter_again",
    the_run_of_th11_started_over_and_its_replay_kept => "the_run_started_over_and_its_replay_kept",
    an_attempt_of_th11s_kept_replay_watched => "an_attempt_of_the_kept_replay_watched",
    a_chapter_of_th11_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    a_run_of_th11_left_in_its_second_chapter => "a_run_left_in_its_second_chapter",
    the_run_of_th11_picked_up_where_it_was_left => "the_run_picked_up_where_it_was_left",
    a_chapter_of_th11s_second_stage_taken_again => "a_chapter_of_the_stage_after_the_first",
    the_run_of_th11_picked_up_in_the_stage_it_walked_into => "the_run_picked_up_in_the_stage_it_walked_into",
    a_normal_mode_run_of_th11_nothing_is_kept_of => "a_normal_mode_run_nothing_is_kept_of",
    the_stage_of_th11_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th11_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th11_further_back_taken_again => "a_chapter_further_back_taken_again",
    a_death_of_th11_answered_by_a_chapter_further_back => "a_death_answered_by_a_chapter_further_back",
    a_death_of_th11_in_the_nonspell_after_a_card_answered_by_that_nonspell => "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    a_run_of_th11_played_to_its_clear_without_a_bomb => "a_run_played_to_its_clear_without_a_bomb",
    a_normal_mode_run_of_th11_played_to_its_clear => "a_normal_mode_run_played_to_its_clear",
    the_extra_of_th11_played_to_its_clear => "the_extra_played_to_its_clear",
    the_extra_of_th11_played_to_its_clear_with_no_hit_and_no_bomb => "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    a_chapter_of_th11s_extra_taken_again => "a_chapter_of_the_extra_taken_again",
    an_attempt_at_a_card_of_th11_counted_and_kept => "an_attempt_at_a_card_counted_and_kept",
    a_normal_run_of_th11_played_to_its_clear_with_no_hit_and_no_bomb => "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
);
