//! **The cases the real th08 is tested with, played against the laid-out th08 first** — the same thing
//! `the_cases_of_th06c_the_real_game_is_tested_with` is for that game, and the reason is the same: a
//! recipe whose step never reaches its point, or a check that asks for something no launch shows, fails
//! here in a quarter of a second instead of costing a launch of the real game.
//!
//! **Which for th08 is the difference between a minute and eight.** The first two of this game's cases
//! were written against the real thing and nothing else — `a_run_started_and_given_up` took 37 seconds of
//! game and `a_normal_mode_run_played_without_a_bomb` 436 — and the second of them was written wrong once: its
//! check looked for a `left the band` line, which orb writes on a hit and not per frame, so the first run
//! of it failed after seven minutes of play on a question the recipe had never asked.
//!
//! **Every case is gated here, each by a test of its own**, which [`CASES`] names and
//! [`every_case_of_th08_has_a_test_of_its_own`] holds to `orb-real`'s own list — a case depending on no other, so
//! each is played the way `xtask real` plays it alone: its own launches first, `Case::before`, from the records it
//! starts from, `Case::starts_from`.
//!
//! **What the laid-out game can say about them and what it cannot.** It can say the recipe runs out: the
//! front end walked by this game's own screen numbers, the run entered, the rules given a field to play
//! in, the stages walked in the order this game's own chooser picks them — see
//! [`A_RUNS_STAGES`](crate::fake::th08) — the bomb held back, the run handed over to the title and the
//! launch quit. It cannot say the addresses are this exe's — a laid-out th08 is written from the same
//! constants `Th08` reads — nor how far a run of the real game the rules get, which is what a launch by
//! hand is for.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, TRACE_FILE, Trace};

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in.
///
/// **Past what `a_normal_mode_run_played_without_a_bomb` takes here**: six stages of [`A_STAGE`] is 27,000,
/// and the 36,000 the recipe then plays of the last stage, with the front end walked and the quit pressed
/// either side of it. That recipe's told places are its cards' own and bracket nothing, so no bound of it is
/// sat out in full the way the chapter brackets it held before were.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing a whole run walks through stages
/// rather than sitting in the first — and short, because six of them stand between this laid-out run and
/// the last stage its recipe waits for, where the real game takes some thirty minutes over the same six.
///
/// **And past stage 1's own midstage table and on to its boss**, who arrives for the stage's last 900
/// frames: a case that plays on into the chapter after the one it was asked in reaches that chapter on the
/// laid-out game only where the stage lasts that long, the laid-out stage having no midboss for it to be.
const A_STAGE: u32 = 4_500;

/// **Every case of th08's, each the test of its own below** — see [`every_case_of_th08_has_a_test_of_its_own`].
const CASES: [&str; 22] = [
    "a_run_started_and_given_up",
    "a_normal_mode_run_played_without_a_bomb",
    "a_death_answered_by_the_chapter_again",
    "the_run_started_over_and_its_replay_kept",
    "an_attempt_of_the_kept_replay_watched",
    "a_chapter_taken_again_without_a_death",
    "a_run_left_in_its_second_chapter",
    "the_run_picked_up_where_it_was_left",
    "a_chapter_of_the_stage_after_the_first",
    "the_run_picked_up_in_the_stage_it_walked_into",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_run_played_to_its_clear_without_a_bomb",
    "a_normal_mode_run_played_to_its_clear",
    "a_normal_mode_run_played_to_kaguyas_clear",
    "the_extra_played_to_its_clear",
    "a_chapter_of_the_extra_taken_again",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "a_stage_practised_on_its_own",
];

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
}

/// **Every case `orb-real` has is named in [`CASES`], and every name there is a case** — which is what makes a
/// case added there reach a test here.
#[test]
fn every_case_of_th08_has_a_test_of_its_own() {
    let names: Vec<&str> = orb_real::cases_of(Game::Th08)
        .iter()
        .map(|case| case.name)
        .collect();
    assert_eq!(
        names, CASES,
        "th08's cases are not the ones this file names"
    );
}

/// What the launch before left under its directory, re-rooted under this one — the trace left out, which is
/// each launch's own.
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

/// **One case played out on the laid-out game as `xtask real` plays it alone**: the records it starts from in
/// this laid-out game's own `score.dat` — see [`Fake::the_score_file_a_case_starts_from`] — then each case it
/// plays first and itself last, each launch finding what the one before left. Without that a case that picks
/// a run up finds nothing to pick up: the first launch of this game's pick-up here read `resume: nothing was
/// left of normal-0-ply00a` where the launch before had just left it.
fn the_case_passes(name: &str) {
    in_its_own_process(|| {
        let case = orb_real::case_of(Game::Th08, name)
            .unwrap_or_else(|| panic!("th08 has no case called {name}"));
        let mut left: Vec<(PathBuf, Vec<u8>)> =
            Fake::the_score_file_a_case_starts_from(case.starts_from)
                .map(|bytes| (PathBuf::from("score.dat"), bytes))
                .into_iter()
                .collect();
        let launches = case.before.iter().map(|before| {
            orb_real::case_of(Game::Th08, before).unwrap_or_else(|| {
                panic!("{name} plays {before} first, which is no case of th08's")
            })
        });
        for launch in launches.chain(std::iter::once(case)) {
            let game = Fake::attach_configured(
                Display::agreed(120),
                &format!("th08-real-{name}-{}", launch.name),
                Work::wandering(WORK_US),
                settings(&launch),
            );
            let dir = game.launch().dir().to_path_buf();
            for (inside, bytes) in &left {
                game.sim().files().put(dir.join(inside), bytes);
            }
            // **The rules given a stage to play in**, which is what these cases are about past the walk: a run
            // whose stage never ran would be a recipe's `play_until` waiting on a clock that stood still, and a
            // field with nothing on it would be a launch with nothing for the word to be about.
            game.plays_its_own_stage();
            game.stages_last(A_STAGE);
            played_out(&game, &dir, &launch);
            left = left_by(&game, &dir);
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
    a_run_of_th08_started_and_given_up => "a_run_started_and_given_up",
    a_normal_mode_run_of_th08_played_without_a_bomb => "a_normal_mode_run_played_without_a_bomb",
    a_death_of_th08_answered_by_the_chapter_again => "a_death_answered_by_the_chapter_again",
    the_run_of_th08_started_over_and_its_replay_kept => "the_run_started_over_and_its_replay_kept",
    an_attempt_of_th08s_kept_replay_watched => "an_attempt_of_the_kept_replay_watched",
    a_chapter_of_th08_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    a_run_of_th08_left_in_its_second_chapter => "a_run_left_in_its_second_chapter",
    the_run_of_th08_picked_up_where_it_was_left => "the_run_picked_up_where_it_was_left",
    a_chapter_of_th08s_stage_after_the_first => "a_chapter_of_the_stage_after_the_first",
    the_run_of_th08_picked_up_in_the_stage_it_walked_into => "the_run_picked_up_in_the_stage_it_walked_into",
    a_normal_mode_run_of_th08_nothing_is_kept_of => "a_normal_mode_run_nothing_is_kept_of",
    a_run_of_th08_played_to_its_clear_without_a_bomb => "a_run_played_to_its_clear_without_a_bomb",
    a_normal_mode_run_of_th08_played_to_its_clear => "a_normal_mode_run_played_to_its_clear",
    a_normal_mode_run_of_th08_played_to_kaguyas_clear => "a_normal_mode_run_played_to_kaguyas_clear",
    the_extra_of_th08_played_to_its_clear => "the_extra_played_to_its_clear",
    a_chapter_of_th08s_extra_taken_again => "a_chapter_of_the_extra_taken_again",
    the_stage_of_th08_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th08_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th08_further_back_taken_again => "a_chapter_further_back_taken_again",
    a_death_of_th08_answered_by_a_chapter_further_back => "a_death_answered_by_a_chapter_further_back",
    a_death_in_the_nonspell_after_a_card_of_th08_answered_by_that_nonspell => "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    a_stage_of_th08_practised_on_its_own => "a_stage_practised_on_its_own",
);
