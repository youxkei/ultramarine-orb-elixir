//! **The cases the real th06nc is tested with, played against the laid-out th06nc first** — the same
//! thing `the_cases_of_th06c_the_real_game_is_tested_with` is for that game, and its doc has why: a recipe
//! whose step never reaches its point, or a check that asks for something no launch shows, fails here
//! before it costs a launch of the real game.
//!
//! **Every case of this game's is gated here, each by a test of its own**, which [`CASES`] names and
//! [`every_case_of_th06nc_has_a_test_of_its_own`] holds to `orb-real`'s own list — a case depending on no other,
//! so each is played the way `xtask real` plays it alone: its own launches first, `Case::before`, from the
//! records it starts from, `Case::starts_from`. The last of them to be gated was the clear and the Extra, which
//! wanted three things of the laid-out game: the stage the Extra's own run starts in, its end reaching the screen
//! a run ends at rather than an ending, and a boss beaten leaving with a few points of life rather than with her
//! attack's own. See `fake::th06nc`, where each is beside the address it was read from.
//!
//! **And the laid-out game is held to walking where the real one walked**, by the traces `cargo xtask real`
//! left where `ORB_REAL_TRACES_TH06NC` names them — the same holding th06c's gate does. Passing its own
//! check is what a case asks of a launch; walking the places the real launch walked, in its order, is what
//! says the laid-out front end and stage are the real ones as far as a recipe can tell. Without the holding
//! a fake that reached a check's fields by another path — a screen skipped, a select the real game repeats
//! a held key on taken as one edge — passed here and cost a launch to find out.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, Start, TRACE_FILE, Trace};

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing into the stage after it can.
const A_STAGE: u32 = 6_000;

/// **Every case of th06nc's, each the test of its own below** — see
/// [`every_case_of_th06nc_has_a_test_of_its_own`].
const CASES: [&str; 21] = [
    "a_death_answered_by_the_chapter_again",
    "the_run_started_over_and_its_replay_kept",
    "an_attempt_of_the_kept_replay_watched",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "a_chapter_taken_again_without_a_death",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_run_left_in_its_second_chapter",
    "the_run_picked_up_where_it_was_left",
    "a_chapter_of_the_stage_after_the_first",
    "the_run_picked_up_in_the_stage_it_walked_into",
    "an_attempt_at_a_card_counted_and_kept",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_stage_practised_on_its_own",
    "a_run_played_to_its_clear",
    "a_run_played_to_its_clear_without_a_bomb",
    "the_extra_played_to_its_clear",
    "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
];

/// **The cases played over a stage with nothing on the field** — see [`the_case_passes`], which has why:
/// a whole game is more than the laid-out rules can dodge their way through.
/// **And the 完全無欠モード clear is not one of them**, where the other two whole-game cases are: what
/// that case is measured by is the deaths, and a field with nothing on it is a run with nothing to die
/// to — a launch of it over an empty stage reached the sixth with `deaths=0` and its own check refused
/// it, which is the check working. Dying is what this one is for, so it is played over a field that
/// fires, the way `a_death_answered_by_the_chapter_again` is.
const WITHOUT_BULLETS: [&str; 4] = [
    "a_run_played_to_its_clear_without_a_bomb",
    "the_extra_played_to_its_clear",
    "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
];

/// What the launch before left under its directory, re-rooted under this one — the trace left out, that
/// being the launch's own record and not a file the game found.
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

/// The session directories `cargo xtask real` left, as `ORB_REAL_TRACES_TH06NC` names them, `;` apart —
/// none where it names none, which is a run of the suite with no real launch to hold the laid-out one to.
fn real_traces() -> Vec<PathBuf> {
    std::env::var(format!(
        "ORB_REAL_TRACES_{}",
        Game::Th06nc.name().to_uppercase()
    ))
    .unwrap_or_default()
    .split(';')
    .filter(|named| !named.is_empty())
    .map(PathBuf::from)
    .collect()
}

/// **The places the laid-out launch walked held against the real one's**, where a session holds a trace
/// of this case — see `Trace::shape` — and where the two part, both walks with the step they parted at
/// marked.
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

/// **Every case `orb-real` has is named in [`CASES`], and every name there is a case** — which is what makes
/// a case added there reach a test here.
#[test]
fn every_case_of_th06nc_has_a_test_of_its_own() {
    let names: Vec<&str> = orb_real::cases_of(Game::Th06nc)
        .iter()
        .map(|case| case.name)
        .collect();
    assert_eq!(
        names, CASES,
        "th06nc's cases are not the ones this file names"
    );
}

/// **One case played out on the laid-out game as `xtask real` plays it alone**: its records holding a clear
/// only where it starts from one, `Case::starts_from`, then each case it plays first and itself last, each
/// launch finding what the one before left.
fn the_case_passes(name: &str) {
    in_its_own_process(|| {
        let against = real_traces();
        let case = orb_real::case_of(Game::Th06nc, name)
            .unwrap_or_else(|| panic!("th06nc has no case called {name}"));
        let a_clear = case.starts_from != Start::Nothing;
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        let launches = case.before.iter().map(|before| {
            orb_real::case_of(Game::Th06nc, before).unwrap_or_else(|| {
                panic!("{name} plays {before} first, which is no case of th06nc's")
            })
        });
        let mut parted = None;
        let mut held = false;
        for launch in launches.chain(std::iter::once(case)) {
            let game = Fake::attach_configured(
                Display::agreed(120),
                &format!("th06nc-real-{name}-{}", launch.name),
                Work::wandering(WORK_US),
                settings(&launch),
            );
            game.stages_last(A_STAGE);
            // **And nothing on the field for the whole-game cases**, whose runs the laid-out rules cannot
            // otherwise play to an end: this stage's bullets are aimed at the player from a corner and the
            // rules took two of them inside stage 1 with the bomb forbidden, so the run was over four stages
            // short of the clear the case is about. What those ask of a launch is the path a clear takes —
            // six stages, the ending, the entry, and no bomb in the word — and the real game's own launch is
            // where the dodging is measured.
            if WITHOUT_BULLETS.contains(&launch.name) {
                game.fires_at_the_player(false);
            }
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
    a_death_of_th06nc_answered_by_the_chapter_again => "a_death_answered_by_the_chapter_again",
    the_run_of_th06nc_started_over_and_its_replay_kept => "the_run_started_over_and_its_replay_kept",
    an_attempt_of_th06ncs_kept_replay_watched => "an_attempt_of_the_kept_replay_watched",
    a_death_in_the_nonspell_after_a_card_of_th06nc_answered_by_that_nonspell => "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    a_chapter_of_th06nc_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    the_stage_of_th06nc_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th06nc_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th06nc_further_back_taken_again => "a_chapter_further_back_taken_again",
    a_death_of_th06nc_answered_by_a_chapter_further_back => "a_death_answered_by_a_chapter_further_back",
    a_run_of_th06nc_left_in_its_second_chapter => "a_run_left_in_its_second_chapter",
    the_run_of_th06nc_picked_up_where_it_was_left => "the_run_picked_up_where_it_was_left",
    a_chapter_of_th06nc_in_the_stage_after_the_first => "a_chapter_of_the_stage_after_the_first",
    the_run_of_th06nc_picked_up_in_the_stage_it_walked_into => "the_run_picked_up_in_the_stage_it_walked_into",
    an_attempt_at_a_card_of_th06nc_is_counted_and_kept => "an_attempt_at_a_card_counted_and_kept",
    a_normal_mode_run_of_th06nc_is_kept_nothing_of => "a_normal_mode_run_nothing_is_kept_of",
    a_stage_of_th06nc_practised_on_its_own => "a_stage_practised_on_its_own",
    a_run_of_th06nc_played_to_its_clear_in_pointdevice_mode => "a_run_played_to_its_clear",
    a_run_of_th06nc_played_to_its_clear_without_a_bomb => "a_run_played_to_its_clear_without_a_bomb",
    the_extra_of_th06nc_played_to_its_clear => "the_extra_played_to_its_clear",
    the_extra_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb => "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
    a_normal_run_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb => "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
);
