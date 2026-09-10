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
//! **Every chain is gated here**, which [`CHAINS`] names and
//! [`every_chain_of_th08s_cases_has_a_test_of_its_own`] holds to `orb-real`'s own list — thirteen of them, one
//! case apiece but for four: the run that died and was retried is started over with its replay kept and the
//! attempt it was rewound from watched, the run left in a chapter is picked up, the run left in stage 2 is
//! picked up there, and Eirin's clear is followed by Kaguya's and the Extra.
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
/// **And past stage 1's own midstage table**, whose last boundary is at script 4086: a case that plays on
/// into the chapter after the one it was asked in reaches that chapter on the laid-out game only where the
/// stage lasts that long.
const A_STAGE: u32 = 4_500;

/// **The case each chain gated here starts at** — see `Case::continues`, and [`the_chain_from`].
const CHAINS: [&str; 13] = [
    "a_run_started_and_given_up",
    "a_normal_mode_run_played_without_a_bomb",
    "a_death_answered_by_the_chapter_again",
    "a_chapter_taken_again_without_a_death",
    "a_run_left_in_its_second_chapter",
    "a_chapter_of_the_stage_after_the_first",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_further_back_taken_again",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_run_played_to_its_clear_without_a_bomb",
    "a_normal_mode_run_played_to_its_clear",
    "a_stage_practised_on_its_own",
];

fn settings(case: &Case) -> impl FnOnce(&mut orb_config::Config) {
    let recipe = case.recipe.to_owned();
    move |config| {
        config.autoplay = Some(recipe);
        config.trace = Some(PathBuf::from(TRACE_FILE));
        config.log_level = LogLevel::Verbose;
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

/// The cases of the chain that starts at `head` — see `orb_real::chains_of`.
///
/// # Panics
/// If no chain starts there, which is a chain this file names and `orb-real` does not have.
fn the_chain_from(head: &str) -> Vec<Case> {
    orb_real::chains_of(Game::Th08)
        .into_iter()
        .find(|chain| chain[0].name == head)
        .unwrap_or_else(|| panic!("th08 has no chain starting at {head}"))
}

/// **Every chain [`CHAINS`] names is a chain `orb-real` has, and every chain it has is named** — which is
/// what makes a case added there reach a test here.
#[test]
fn every_chain_of_th08s_cases_has_a_test_of_its_own() {
    let heads: Vec<&str> = orb_real::chains_of(Game::Th08)
        .iter()
        .map(|chain| chain[0].name)
        .collect();
    let mut sorted = heads.clone();
    sorted.sort_unstable();
    let mut wanted = CHAINS;
    wanted.sort_unstable();
    assert_eq!(
        sorted, wanted,
        "the chains th08's cases make are not the ones this file names: {heads:?}",
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

/// One chain played out on the laid-out game, in its own order — **each case in a directory of its own, with
/// what the case before left put into it where the case continues that one**, which is `Case::continues`
/// and what `xtask real` does with the copy's directory. Without it a case that picks a run up finds nothing
/// to pick up: the first launch of this game's pick-up here read `resume: nothing was left of
/// normal-0-ply00a` where the case before had just left it.
fn the_chain_passes(head: &str) {
    in_its_own_process(|| {
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        for case in the_chain_from(head) {
            let name = format!("th08-real-{}", case.name);
            let game = Fake::attach_configured(
                Display::agreed(120),
                &name,
                Work::wandering(WORK_US),
                settings(&case),
            );
            let dir = game.launch().dir().to_path_buf();
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
            }
            // **The rules given a stage to play in**, which is what both of these cases are about past
            // the walk: a run whose stage never ran would be a recipe's `play_until` waiting on a clock
            // that stood still, and a field with nothing on it would be a launch with nothing for the
            // word to be about.
            game.plays_its_own_stage();
            game.stages_last(A_STAGE);
            played_out(&game, &dir, &case);
            left = left_by(&game, &dir);
        }
    });
}

/// **A run started by the recipe and handed back to the game's own title**, which is what
/// `Th08::leave_run` writes and what the first attempt at it faulted on: the scene the run asks for, the
/// two bits of its kind, and the decide the hand-over waits for.
///
/// What this says over `the_front_end_of_th08_walked` is the way *out* of a run — that the two writes
/// reach the manager's own ending section, that the press it waits for is the one orb's retry already
/// makes, and that the front end comes back up after it.
#[test]
fn a_run_of_th08_started_and_given_up() {
    the_chain_passes("a_run_started_and_given_up");
}

/// **And a Normal run played out by the rules with the bomb forbidden**, which is the launch that says the
/// arrays a bot of this game is told of are arrays it can play against: the enemies, the bullets with the
/// step their own kind takes, the lasers, the items and the boss.
///
/// **What the laid-out game asks of it is the walk, the word and the bomb** — the difficulty the select
/// was left on, a stage clock past two thousand frames, a direction in the word inside the run and no
/// bomb bit anywhere in it. How far into a run of the *real* game the rules get is the real game's to
/// say, and `checks::a_normal_mode_run_of_th08_played_without_a_bomb` asks nothing of it for the reason
/// written there: one launch cleared stage 1 with no death and the next lost three lives in it.
///
/// **And the recipe's own wait for the last stage is answered here by the stage walk**, which is what
/// `A_RUNS_STAGES` is for: a laid-out run stepping 0, 1, 2, 4, 5, 6 reaches the `stage: 7` that step waits
/// for, where one counting 0 through 5 never would.
#[test]
fn a_normal_mode_run_of_th08_played_without_a_bomb() {
    the_chain_passes("a_normal_mode_run_played_without_a_bomb");
}

/// **And the mode's own case: a chapter of this game taken, died in and put back** — then the run started
/// over with its replay kept, and the attempt of that replay the death rewound the run from watched off
/// orb's own list behind the title's fifth item.
///
/// What the laid-out game can say about it is the whole path bar the addresses — orb's own mode question
/// answered over the title, the run entered, the chapter taken, the death counted, the retry answered
/// with the chapter again, the clock back and the run playing on, and a stretch of the run no retry kept
/// played back into a run built out of the file. What it cannot say is that a restore of the *real*
/// game's twenty-one megabytes and five heap regions leaves it running, which is what the launch in
/// `checks::a_death_of_th08_answered_by_the_chapter_again`'s own note measured.
#[test]
fn a_death_of_th08_answered_by_the_chapter_again_and_the_attempt_it_ended_watched() {
    the_chain_passes("a_death_answered_by_the_chapter_again");
}

// And one test for each of the other chains, so that they are played at once rather than one after another —
// see [`CHAINS`]. Named after the case each starts at, so a failure names the chain in the harness's own
// output.

#[test]
fn a_chapter_of_th08_taken_again_without_a_death() {
    the_chain_passes("a_chapter_taken_again_without_a_death");
}

/// **The run left in its second chapter, and picked up where it was left.**
#[test]
fn a_run_of_th08_left_in_its_second_chapter_and_picked_up() {
    the_chain_passes("a_run_left_in_its_second_chapter");
}

#[test]
fn a_chapter_of_th08s_stage_after_the_first_and_the_run_picked_up_in_it() {
    the_chain_passes("a_chapter_of_the_stage_after_the_first");
}

#[test]
fn the_stage_of_th08_taken_again_from_its_start() {
    the_chain_passes("the_stage_taken_again_from_its_start");
}

#[test]
fn the_run_of_th08_given_up_at_the_retry() {
    the_chain_passes("the_run_given_up_at_the_retry");
}

#[test]
fn a_chapter_of_th08_further_back_taken_again() {
    the_chain_passes("a_chapter_further_back_taken_again");
}

#[test]
fn a_normal_mode_run_of_th08_nothing_is_kept_of() {
    the_chain_passes("a_normal_mode_run_nothing_is_kept_of");
}

/// **And the whole game played to its clear**, which asks of the laid-out game what the real one's clear
/// has: the six stages walked, the last one ending on its Last Spell — see `fake::th08`'s `THE_LAST_SPELL` —
/// and the ending after it, with no bomb pressed on the way.
#[test]
fn a_run_of_th08_played_to_its_clear_without_a_bomb() {
    the_chain_passes("a_run_played_to_its_clear_without_a_bomb");
}

/// **And the Extra, behind the two Normal runs of the game's own mode that light it** — Eirin's clear, the
/// next run routed to Kaguya by the records that one left, and the Extra on the title item Kaguya's ending
/// lit: which asks of the laid-out game the records its title and its chooser read — see `fake::th08`'s
/// `Records` — carried from one case to the next in the directory they share.
#[test]
fn a_normal_mode_run_of_th08_played_to_its_clear_and_the_extra_the_two_unlocked() {
    the_chain_passes("a_normal_mode_run_played_to_its_clear");
}

/// **And one stage practised on its own** — the title's fourth item and its three screens, the run reading
/// as a practice, orb keeping nothing of it, and a chapter of it taken again all the same.
#[test]
fn a_stage_of_th08_practised_on_its_own() {
    the_chain_passes("a_stage_practised_on_its_own");
}
