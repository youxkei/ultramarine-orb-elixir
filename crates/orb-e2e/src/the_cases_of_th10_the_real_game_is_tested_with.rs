//! **The cases the real th10 is tested with, played against the laid-out th10 first** — the same thing
//! `the_cases_of_th08_the_real_game_is_tested_with` is for that game, and the reason is the same: a recipe whose
//! step never reaches its point, or a check that asks for something no launch shows, fails here in seconds
//! instead of costing a launch of the real game.
//!
//! **Every chain is gated here**, which [`CHAINS`] names and
//! [`every_chain_of_th10s_cases_has_a_test_of_its_own`] holds to `orb-real`'s own list — one case apiece but for
//! the death answered by the chapter again, whose run is started over with its replay kept and the attempt it was
//! rewound from watched, the stage practised after the run that cleared it, and the run left in its second
//! chapter, which is picked up.
//!
//! **What the laid-out game can say about them and what it cannot.** It can say the recipe runs out: the title
//! walked by this game's own screen numbers, the run entered, the rules given a field to play in, the stages
//! walked one after another, the bomb held back, the run handed over to the title and the launch quit. It cannot
//! say the addresses are this exe's — a laid-out th10 is written from the same constants `Th10` reads — nor how
//! far a run of the real game the rules get, which is what a launch by hand is for.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, TRACE_FILE, Trace};

use crate::fake::th10::{Fake, THE_GAMES_SCORE_FILE};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::game::th10::score_file;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in: six stages of [`A_STAGE`] and the
/// recipes' own waits either side of them.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing a whole run walks through stages rather
/// than sitting in the first — and past its midboss, whose two attacks from frame 1200 are a stage's second and
/// third chapters, and on to its boss for the last 900 frames: the chapters these cases are over.
const A_STAGE: u32 = 4_500;

/// **The case each chain gated here starts at** — see `Case::continues`, and [`the_chain_from`].
const CHAINS: [&str; 16] = [
    "a_run_started_and_given_up",
    "a_normal_mode_run_played_without_a_bomb",
    "a_death_answered_by_the_chapter_again",
    "a_chapter_taken_again_without_a_death",
    "a_run_left_in_its_second_chapter",
    "a_chapter_of_the_stage_after_the_first",
    "a_normal_mode_run_nothing_is_kept_of",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "a_run_played_to_its_clear_without_a_bomb",
    "a_normal_mode_run_played_to_its_clear",
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

/// The cases of the chain that starts at `head` — see `orb_real::chains_of`.
///
/// # Panics
/// If no chain starts there, which is a chain this file names and `orb-real` does not have.
fn the_chain_from(head: &str) -> Vec<Case> {
    orb_real::chains_of(Game::Th10)
        .into_iter()
        .find(|chain| chain[0].name == head)
        .unwrap_or_else(|| panic!("th10 has no chain starting at {head}"))
}

/// **Every chain [`CHAINS`] names is a chain `orb-real` has, and every chain it has is named** — which is what
/// makes a case added there reach a test here.
#[test]
fn every_chain_of_th10s_cases_has_a_test_of_its_own() {
    let heads: Vec<&str> = orb_real::chains_of(Game::Th10)
        .iter()
        .map(|chain| chain[0].name)
        .collect();
    let mut sorted = heads.clone();
    sorted.sort_unstable();
    let mut wanted = CHAINS;
    wanted.sort_unstable();
    assert_eq!(
        sorted, wanted,
        "the chains th10's cases make are not the ones this file names: {heads:?}",
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

/// **And what it left beside itself**: the files opened by a bare name, which land in the directory the game runs
/// in — `scoreth10.dat`, which the game reads at its startup and writes at every run's teardown, and orb's fork of
/// it. The copy the real cases run on is that directory, so a case that continues another finds the records the
/// one before wrote; without them a clear written there lights nothing on the launch after it.
fn left_beside(game: &Fake) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter(|path| path.components().count() == 1)
        .filter_map(|path| Some((path.clone(), files.get(&path)?)))
        .collect()
}

/// **The score file of a copy that has played a Normal run of Reimu A into its first stage**, which is what the
/// copies the real cases run on are, and what offers that stage's practice: the run reaching a stage sets the
/// byte at `+0x5` of its entry (0x416c08), the entries from `+0x4dc` of the shot type's record by `difficulty * 6
/// + stage`, and the practice select takes a decide only on a stage whose byte is set (0x43114f).
///
/// The record is summed again after the byte, as the game's parse checks every record's sum (0x42b0a5 through
/// 0x42b109).
fn a_score_file_that_has_reached_stage_1() -> Vec<u8> {
    const NORMAL_STAGE_1_OFFERED: usize = 0x4dc + 6 * 8 + 0x5;
    let mut record = score_file::shot_types_record(0, 0);
    record[NORMAL_STAGE_1_OFFERED] = 1;
    let sum = record[0x8..]
        .iter()
        .fold(0u32, |sum, byte| sum.wrapping_add(u32::from(*byte)));
    record[0x4..0x8].copy_from_slice(&sum.to_le_bytes());
    let mut records = score_file::Records::default();
    records.shot_types[0] = Some(record);
    score_file::encode(&records)
}

/// One chain played out on the laid-out game, in its own order — **each case in a directory of its own, with
/// what the case before left put into it where the case continues that one**, which is `Case::continues` and
/// what `xtask real` does with the copy's directory.
fn the_chain_passes(head: &str) {
    in_its_own_process(|| {
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        let mut beside: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        for case in the_chain_from(head) {
            let name = format!("th10-real-{}", case.name);
            let game = Fake::attach_configured(
                Display::agreed(120),
                &name,
                Work::wandering(WORK_US),
                settings(&case),
            );
            let dir = game.launch().dir().to_path_buf();
            game.sim().files().put(
                THE_GAMES_SCORE_FILE,
                a_score_file_that_has_reached_stage_1(),
            );
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
                for (name, bytes) in &beside {
                    game.sim().files().put(name, bytes);
                }
            }
            // **The rules given a stage to play in**, which is what these cases are about past the walk: a run
            // whose stage never ran would be a recipe's `play_until` waiting on a clock that stood still.
            game.plays_its_own_stage();
            game.stages_last(A_STAGE);
            played_out(&game, &dir, &case);
            left = left_by(&game, &dir);
            beside = left_beside(&game);
        }
    });
}

/// **A run started by the recipe and handed back to the game's own title**, which is what `Th10::leave_run`
/// writes: the scene wanted at the title, which the supervisor answers by taking the run down.
#[test]
fn a_run_of_th10_started_and_given_up() {
    the_chain_passes("a_run_started_and_given_up");
}

/// **And a Normal run played out by the rules with the bomb forbidden**, which is the launch that says the arrays
/// a bot of this game is told of are arrays it can play against — and one stage practised on its own after it,
/// the main menu's item 2 and the screens after it offering the stage that run cleared, the run reading as a
/// practice, and orb keeping nothing of it.
#[test]
fn a_normal_mode_run_of_th10_played_without_a_bomb() {
    the_chain_passes("a_normal_mode_run_played_without_a_bomb");
}

/// **And the mode's own case: a chapter of this game taken, died in and put back** — and that run started over
/// with its replay kept, and the attempt the death rewound it from watched.
#[test]
fn a_death_of_th10_answered_by_the_chapter_again() {
    the_chain_passes("a_death_answered_by_the_chapter_again");
}

// And one test for each of the other chains, so that they are played at once rather than one after another —
// see [`CHAINS`]. Named after the case each starts at, so a failure names the chain in the harness's own output.

#[test]
fn a_chapter_of_th10_taken_again_without_a_death() {
    the_chain_passes("a_chapter_taken_again_without_a_death");
}

/// **The run left in its second chapter, and picked up where it was left.**
#[test]
fn a_run_of_th10_left_in_its_second_chapter_and_picked_up() {
    the_chain_passes("a_run_left_in_its_second_chapter");
}

/// **A chapter of the stage after the first taken again and played past, and the run left there picked up in
/// that stage.**
#[test]
fn a_chapter_of_th10s_second_stage_taken_again_and_picked_up_there() {
    the_chain_passes("a_chapter_of_the_stage_after_the_first");
}

#[test]
fn a_normal_mode_run_of_th10_nothing_is_kept_of() {
    the_chain_passes("a_normal_mode_run_nothing_is_kept_of");
}

#[test]
fn the_stage_of_th10_taken_again_from_its_start() {
    the_chain_passes("the_stage_taken_again_from_its_start");
}

#[test]
fn the_run_of_th10_given_up_at_the_retry() {
    the_chain_passes("the_run_given_up_at_the_retry");
}

#[test]
fn a_chapter_of_th10_further_back_taken_again() {
    the_chain_passes("a_chapter_further_back_taken_again");
}

/// And the same list over a death, the question the hit raised answered by the recipe and not the rules.
#[test]
fn a_death_of_th10_answered_by_a_chapter_further_back() {
    the_chain_passes("a_death_answered_by_a_chapter_further_back");
}

/// **And a death in the attack a boss moves to after a card**, put back at that attack's own start.
#[test]
fn a_death_of_th10_in_the_nonspell_after_a_card_answered_by_that_nonspell() {
    the_chain_passes("a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell");
}

/// **And the whole game played to its clear**: the six stages walked and the ending after the last, with no
/// bomb pressed on the way, and the name typed into the clear's own entry, the title's screen 15 (0x432cb0).
#[test]
fn a_run_of_th10_played_to_its_clear_without_a_bomb() {
    the_chain_passes("a_run_played_to_its_clear_without_a_bomb");
}

/// **And the Extra, behind the Normal run of the game's own mode that lights it**: that run cleared and its
/// records left in the directory, the main menu's item 1 offered on the launch after it and played to the
/// Extra's own end, the Extra played to its clear with no hit and no bomb on the launch after that, and a chapter
/// of the Extra taken again last.
#[test]
fn the_extra_of_th10_played_to_its_clear_behind_the_normal_run_that_lit_it() {
    the_chain_passes("a_normal_mode_run_played_to_its_clear");
}

/// **Two runs in one launch, each taking a chapter of the same card again**: the second's count is the first's
/// with the card's start and the retry on top.
#[test]
fn an_attempt_at_a_card_of_th10_counted_and_kept() {
    the_chain_passes("an_attempt_at_a_card_counted_and_kept");
}

/// **And the Normal run played to its ending with no hit and no bomb**, in the mode.
#[test]
fn a_normal_run_of_th10_played_to_its_clear_with_no_hit_and_no_bomb() {
    the_chain_passes("a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb");
}
