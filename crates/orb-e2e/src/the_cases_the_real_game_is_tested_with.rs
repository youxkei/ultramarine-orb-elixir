//! **The cases the real game is tested with, played against the laid-out games first.**
//!
//! `orb-real` is a list of recipes and, for each, a check over the trace the launch writes; `cargo xtask real`
//! plays them on a copy of the install. What that costs is the game, the machine and the minutes, so a recipe
//! whose step never reaches its point, or a check that asks for something no launch shows, is worth finding
//! before any of that is spent — and the laid-out games are where it is found. Each case here is the same
//! recipe handed to the same reader, with `--trace` on, and the same check over what came out.
//!
//! **And the other way round, this is what holds the laid-out games to the real one.** A recipe written for
//! the real front end walks it by pressing toward orb's own questions and a stage's own chapters; a laid-out
//! game those presses do not walk is a laid-out game that is not the game's front end, and this is where
//! that shows.
//!
//! A case that continues the one before it finds what the earlier launch left: the files that launch wrote
//! under its own directory, put under the next launch's before it starts — which is what sharing a directory
//! across two launches of the real game comes to.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, TRACE_FILE, Trace};

use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over th07 declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in, which is past the sum of every bound
/// its steps carry: a recipe that has not quit by then is one whose step never gave up.
///
/// Past what the case that plays a whole game needs, which is six stages of [`A_STAGE`], the ending after
/// them, and the ten minutes th07's own case gives the walk off the screens a cleared run arrives on — the
/// longest thing here by a factor of six. That ten minutes is for the staff roll the real game plays there
/// and is what the laid-out game spends in full: the screen a cleared run ends at offers a replay and is
/// walked off by the game's own menu on the real one, where here nothing but orb's own write past that offer
/// moves it — so the case's own bound is what ends the step, and this has to be past it.
const FRAMES: u32 = 100_000;

/// **How long a stage of a laid-out game lasts here**, which is what makes a stage transition something a
/// case can play into: nothing said is a stage that never ends, which is what every e2e test about one
/// stage wants and what a case reaching the second cannot have.
///
/// Past every frame the other cases play, so that the stage they are in is the stage they end in: the
/// longest of them plays a run for 1,800 frames after a walk-in of 600. A real stage is about this long,
/// which is what makes the number a stage's rather than a bound's.
const A_STAGE: u32 = 6_000;

/// And how long the ending laid out for a cleared run runs, in the waits its script and the staff roll's own
/// hold. Nothing of a case is about their length — what it reads is that the ending was run out to where the
/// roll begins — so they are as short as saying so allows.
const ENDING_WAITS: i32 = 10;
const ROLL_WAITS: i32 = 10;

/// What the launch before left under its directory, re-rooted under this one.
///
/// **The trace is not one of them**, which is the one file here that is the session's record of a launch
/// rather than something the launch left for the next: carried across, it arrived in the next launch's
/// directory under the name that launch writes its own trace to, and the check then read a run out of the
/// launch before — `the_extra_a_cleared_run_unlocked` read the difficulty of the run that cleared the game
/// instead of the Extra's. On the real game the two are different directories already: `cargo xtask real`
/// keeps each case's trace under the case and points `--keep-in` at the chain's own.
fn left_by(game: &impl Launched, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
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

/// What holding a laid-out launch against the real game's own came to.
enum Held {
    /// No session was named, or it holds no trace of this case.
    NotCompared,
    /// The two walked the same places.
    Same,
    /// They did not, and this is where and what both did.
    Different(String),
}

/// The recipe run out — the game asked to close by its last step — and the trace read back, checked, and
/// held against the real game's own where a session of them was named.
fn played_out(game: &impl Launched, dir: &Path, case: &Case, against: &[PathBuf]) -> Held {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            break;
        }
        game.frame();
    }
    // Said with what the launch did rather than with the count alone: which step it stood on is the whole
    // of what a recipe that never quit has to say for itself.
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
    // **The trace's last chunk, which on a real host the loader writes.** A launch that ends by the game's own
    // last title item is one nothing asked orb to close, so what flushes the chunk there is
    // `DLL_PROCESS_DETACH` — and a laid-out launch has no such moment, the process it runs in being this
    // suite's. Here rather than in either laid-out game, because what it stands for is the process going.
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

/// **The real game's own traces to hold a laid-out launch against**: a session `cargo xtask real` left,
/// named for the game in the environment as `ORB_REAL_TRACES_TH06` or `ORB_REAL_TRACES_TH07`.
///
/// Nothing unless one is named, and that is not tidiness: a trace is megabytes and lives beside the copy of
/// the install it was played in, which a checkout has none of.
///
/// **What it names is a path of the game's own machine**, this suite being built for the target the game runs
/// on and run as a process of it: a path of the host's is one it cannot open. That is what the count in each
/// test is for — a session named and no case held against it is the wrong path rather than a clean run, and
/// it passed silently until the count was there.
///
/// **Several sessions, `;` between them, and the first of them that holds a case is the one that case is held
/// against.** A session holds the cases it was told to play, so a case played again on its own after its
/// recipe changed is a session of one — and the run before it is a session whose trace of that case was
/// played from a recipe that no longer exists. Named ahead of the older session, the newer one answers for
/// the cases it holds and the older one for the rest.
fn real_traces(game: Game) -> Vec<PathBuf> {
    std::env::var(format!("ORB_REAL_TRACES_{}", game.name().to_uppercase()))
        .unwrap_or_default()
        .split(';')
        .filter(|named| !named.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// **The laid-out launch held against the real game's own**, which is what says the laid-out game is walked
/// where the real one is: [`Trace::shape`] — the questions orb put up, the screens its front end went
/// through and the recipe's own steps — since every number underneath those is a laid-out game's to differ
/// in by design.
///
/// A case no named session holds a trace of is passed over rather than failed, a session holding the cases it
/// was told to play; and a difference comes back rather than being raised, so that one run says which of the
/// cases diverged and not only the first of them.
///
/// **Both walks are printed whole**, the difference marked: which place they parted at says little without
/// what each did on either side of it, and a shape is a few dozen lines.
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

/// **The case each chain starts at**, which is every case that continues nothing — see
/// `orb_real::chains_of`, which is what makes one, and [`every_chain_has_a_test_of_its_own`], which holds
/// this list against it so a case added there reaches a test here.
///
/// **The same eleven for both games**, these two and th06c sharing `the_cases_of_a_game_orb_rewinds`.
///
/// **A test per chain and not one test per game**, which is the whole reason this list exists: the sixteen
/// cases were one test apiece and a chain has to be played in its own order — a case that continues is
/// handed the files the one before it left — so all sixteen of each game ran one after another while six of
/// the run's eight threads sat idle. Measured: these two tests alone took 194s of the 32-bit run, and the
/// same split of th06c's took its own 181s to 83s. What the wall clock is now is the longest chain, which is
/// the Normal clear and the Extra that continues it.
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

/// The cases of `game`'s chain that starts at `head`.
///
/// # Panics
/// If no chain starts there, which is a chain this file names and `orb-real` does not have.
fn the_chain_from(game: Game, head: &str) -> Vec<Case> {
    orb_real::chains_of(game)
        .into_iter()
        .find(|chain| chain[0].name == head)
        .unwrap_or_else(|| panic!("{} has no chain starting at {head}", game.name()))
}

/// One `#[test]` per chain, each calling the function that plays one of that game's.
macro_rules! a_test_per_chain {
    ($plays:ident, $($test:ident => $head:literal,)*) => {
        $(
            #[test]
            fn $test() {
                $plays($head);
            }
        )*
    };
}

#[test]
fn every_chain_has_a_test_of_its_own() {
    for game in [Game::Th06, Game::Th07] {
        let heads: Vec<&str> = orb_real::chains_of(game)
            .iter()
            .map(|chain| chain[0].name)
            .collect();
        assert_eq!(
            heads,
            CHAINS,
            "the chains this file plays are not the chains {}'s cases make",
            game.name(),
        );
    }
}

fn the_chain_of_th06_passes(head: &str) {
    use crate::fake::th06::{Fake, the_run};
    in_its_own_process(|| {
        let against = real_traces(Game::Th06);
        let mut held = 0;
        let mut parted: Vec<String> = Vec::new();
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        // **Whether the records the next launch starts from hold a clear**, folded over the chain the way
        // `xtask real` folds it: cleared before every case that does not continue the one before it, and set
        // by a case whose own run leaves one — see `orb_real::Case::leaves_a_clear`.
        let mut a_clear = false;
        let chain = the_chain_from(Game::Th06, head);
        let names: Vec<&str> = chain.iter().map(|case| case.name).collect();
        for case in chain {
            a_clear = a_clear && case.continues;
            let name = format!("th06-real-{}", case.name);
            let game = Fake::attach(&name, the_run(), settings(&case));
            // A stage that shoots, which the real one does and the laid-out one does only where asked: a
            // recipe told to take hits has to have something on the field to be hit by.
            game.fires_at_the_player();
            // And the screens the real game walks between its title and a run, which these recipes walk: the
            // step that aims at the difficulty select is what says so, `done` on the real game where a
            // laid-out one without them has nothing for it to reach.
            game.walks_the_screens_between_its_title_and_a_run();
            // And the records this launch starts from holding no clear unless the case before it left one,
            // which is what `Records` leaves the real copy with — a title offering an Extra the real one does
            // not is a title walked somewhere else. See `Case::leaves_a_clear`.
            if !a_clear {
                game.its_records_hold_no_clear();
            }
            // And a stage that ends, which is what a case playing into the stage after it needs.
            game.stages_last(A_STAGE);
            // And an ending for the run that plays every stage to reach one: a script of a few waits with
            // the staff roll's own after it, which is what orb's skip stops at. The lengths are the game's
            // and these are this laid-out one's — short enough that the case does not spend its frames here.
            game.lays_out_an_ending(ENDING_WAITS, ROLL_WAITS);
            let dir = game.launch().dir().to_path_buf();
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
            }
            a_clear |= case.leaves_a_clear();
            match played_out(&*game, &dir, &case, &against) {
                Held::NotCompared => {}
                Held::Same => held += 1,
                Held::Different(how) => {
                    held += 1;
                    parted.push(how);
                }
            }
            left = left_by(&*game, &dir);
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

a_test_per_chain!(
    the_chain_of_th06_passes,
    a_run_of_th06_left_in_its_second_chapter_and_picked_up => "a_run_left_in_its_second_chapter",
    a_death_of_th06_answered_by_the_chapter_and_the_replay_kept => "a_death_answered_by_the_chapter_again",
    a_chapter_of_th06_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    a_chapter_of_th06_further_back_taken_again => "a_chapter_further_back_taken_again",
    the_stage_of_th06_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th06_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th06s_stage_after_the_first_and_the_run_picked_up_in_it => "a_chapter_of_the_stage_after_the_first",
    a_stage_of_th06_practised_on_its_own => "a_stage_practised_on_its_own",
    a_normal_mode_run_of_th06_nothing_is_kept_of => "a_normal_mode_run_nothing_is_kept_of",
    a_run_of_th06_played_to_its_clear => "a_run_played_to_its_clear",
    a_normal_mode_run_of_th06_played_to_its_clear_and_the_extra_it_unlocked => "a_normal_mode_run_played_to_its_clear",
);

fn the_chain_of_th07_passes(head: &str) {
    use crate::fake::th07::Fake;
    in_its_own_process(|| {
        let against = real_traces(Game::Th07);
        let mut held = 0;
        let mut parted: Vec<String> = Vec::new();
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        // The same fold as th06's above, and for the same reason.
        let mut a_clear = false;
        let chain = the_chain_from(Game::Th07, head);
        let names: Vec<&str> = chain.iter().map(|case| case.name).collect();
        for case in chain {
            a_clear = a_clear && case.continues;
            let name = format!("th07-real-{}", case.name);
            let game = Fake::attach_configured(
                Display::agreed(120),
                &name,
                Work::wandering(WORK_US),
                settings(&case),
            );
            game.streams_its_song(0);
            // And a shot select whose decide starts a run whatever brought that screen up, which the real one
            // does and the laid-out one does only where asked: the attempt out of a replay is watched in a
            // run orb asks that screen for, and a game that starts none is a game where nothing is watched.
            game.its_shot_select_starts_a_run_however_it_came_up();
            // And a stage that ends, which is what a case playing into the stage after it needs.
            game.stages_last(A_STAGE);
            // And the records this launch starts from holding no clear unless the case before it left one —
            // the same reading as th06's above, and the same reason.
            if !a_clear {
                game.its_records_hold_no_clear();
            }
            let dir = game.launch().dir().to_path_buf();
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
            }
            a_clear |= case.leaves_a_clear();
            match played_out(&*game, &dir, &case, &against) {
                Held::NotCompared => {}
                Held::Same => held += 1,
                Held::Different(how) => {
                    held += 1;
                    parted.push(how);
                }
            }
            left = left_by(&*game, &dir);
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

a_test_per_chain!(
    the_chain_of_th07_passes,
    a_run_of_th07_left_in_its_second_chapter_and_picked_up => "a_run_left_in_its_second_chapter",
    a_death_of_th07_answered_by_the_chapter_and_the_replay_kept => "a_death_answered_by_the_chapter_again",
    a_chapter_of_th07_taken_again_without_a_death => "a_chapter_taken_again_without_a_death",
    a_chapter_of_th07_further_back_taken_again => "a_chapter_further_back_taken_again",
    the_stage_of_th07_taken_again_from_its_start => "the_stage_taken_again_from_its_start",
    the_run_of_th07_given_up_at_the_retry => "the_run_given_up_at_the_retry",
    a_chapter_of_th07s_stage_after_the_first_and_the_run_picked_up_in_it => "a_chapter_of_the_stage_after_the_first",
    a_stage_of_th07_practised_on_its_own => "a_stage_practised_on_its_own",
    a_normal_mode_run_of_th07_nothing_is_kept_of => "a_normal_mode_run_nothing_is_kept_of",
    a_run_of_th07_played_to_its_clear => "a_run_played_to_its_clear",
    a_normal_mode_run_of_th07_played_to_its_clear_and_the_extra_it_unlocked => "a_normal_mode_run_played_to_its_clear",
);
