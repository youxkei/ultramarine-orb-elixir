//! **The cases the real th06nc is tested with, played against the laid-out th06nc first** — the same
//! thing `the_cases_of_th06c_the_real_game_is_tested_with` is for that game, and its doc has why: a recipe
//! whose step never reaches its point, or a check that asks for something no launch shows, fails here
//! before it costs a launch of the real game.
//!
//! **All eleven of this game's chains are gated here**, which [`CHAINS`] names and
//! [`every_chain_of_th06ncs_cases_has_a_test_of_its_own`] holds to `orb-real`'s own list — the same as
//! th06c's eleven. The last of them to be gated was the clear and the Extra that continues it, which
//! wanted three things of the laid-out game: the stage the Extra's own run starts in, its end reaching
//! the screen a run ends at rather than an ending, and a boss beaten leaving with a few points of life
//! rather than with her attack's own. See `fake::th06nc`, where each is beside the address it was read
//! from.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_real::{Case, Game, TRACE_FILE, Trace};

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The most frames a case is given to run its recipe out and quit in.
const FRAMES: u32 = 100_000;

/// How long a stage of this laid-out game lasts, so that a case playing into the stage after it can.
const A_STAGE: u32 = 6_000;

/// **The case each chain gated here starts at** — see `Case::continues`, and [`the_chain_from`].
const CHAINS: [&str; 15] = [
    "a_death_answered_by_the_chapter_again",
    "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
    "an_attempt_at_a_card_counted_and_kept",
    "a_chapter_further_back_taken_again",
    "a_death_answered_by_a_chapter_further_back",
    "a_run_left_in_its_second_chapter",
    "a_chapter_taken_again_without_a_death",
    "the_stage_taken_again_from_its_start",
    "the_run_given_up_at_the_retry",
    "a_chapter_of_the_stage_after_the_first",
    "a_normal_mode_run_nothing_is_kept_of",
    "a_stage_practised_on_its_own",
    "a_run_played_to_its_clear_without_a_bomb",
    "a_run_played_to_its_clear",
    "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
];

/// **The cases played over a stage with nothing on the field** — see [`the_chain_passes`], which has why:
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
}

/// The cases of the chain that starts at `head` — see `orb_real::chains_of`.
///
/// # Panics
/// If no chain starts there, which is a chain this file names and `orb-real` does not have.
fn the_chain_from(head: &str) -> Vec<Case> {
    orb_real::chains_of(Game::Th06nc)
        .into_iter()
        .find(|chain| chain[0].name == head)
        .unwrap_or_else(|| panic!("th06nc has no chain starting at {head}"))
}

/// **Every chain [`CHAINS`] names is a chain `orb-real` has, and every chain it has is named** — which is
/// what makes a case added there reach a test here.
#[test]
fn every_chain_of_th06ncs_cases_has_a_test_of_its_own() {
    let heads: Vec<&str> = orb_real::chains_of(Game::Th06nc)
        .iter()
        .map(|chain| chain[0].name)
        .collect();
    let mut sorted = heads.clone();
    sorted.sort_unstable();
    let mut wanted = CHAINS;
    wanted.sort_unstable();
    assert_eq!(
        sorted, wanted,
        "the chains th06nc's cases make are not the ones this file names: {heads:?}",
    );
}

/// One chain played out on the laid-out game, in its own order and its own directory.
fn the_chain_passes(head: &str) {
    in_its_own_process(|| {
        let mut left: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        // **Whether the records the next launch starts from hold a clear**, folded over the chain the way
        // `xtask real` folds it: cleared before a case that does not continue the one before it, and set by
        // a case whose own run leaves one. **Which is what says the Extra's case follows the clear that
        // unlocks it** — see `orb_real::Case::leaves_a_clear`, whose doc has the session this game's own
        // order cost.
        let mut a_clear = false;
        for case in the_chain_from(head) {
            a_clear = a_clear && case.continues;
            let name = format!("th06nc-real-{}", case.name);
            let game = Fake::attach_configured(
                Display::agreed(120),
                &name,
                Work::wandering(WORK_US),
                settings(&case),
            );
            game.stages_last(A_STAGE);
            // **And nothing on the field for the two whole-game cases**, whose runs the laid-out rules
            // cannot otherwise play to an end: this stage's bullets are aimed at the player from a corner
            // and the rules took two of them inside stage 1 with the bomb forbidden, so the run was over
            // four stages short of the clear the case is about. What those two ask of a launch is the path
            // a clear takes — six stages, the ending, the entry, and no bomb in the word — and the real
            // game's own launch is where the dodging is measured.
            if WITHOUT_BULLETS.contains(&case.name) {
                game.fires_at_the_player(false);
            }
            if !a_clear {
                game.its_records_hold_no_clear();
            }
            a_clear |= case.leaves_a_clear();
            let dir = game.launch().dir().to_path_buf();
            if case.continues {
                for (inside, bytes) in &left {
                    game.sim().files().put(dir.join(inside), bytes);
                }
            }
            played_out(&game, &dir, &case);
            left = left_by(&game, &dir);
        }
    });
}

/// **The mode's own case**: a death in this game's one chapter answered by that chapter again, and the run
/// carrying on out of it. Which is the case chapter 1 alone can reach — see `orb_real::cases_of`.
#[test]
fn a_death_of_th06nc_answered_by_the_chapter_again() {
    the_chain_passes("a_death_answered_by_the_chapter_again");
}

/// And the same question raised by the recipe rather than by a death, answered with the chapter again:
/// the chapter comes back with every life still in hand.
#[test]
fn a_chapter_of_th06nc_taken_again_without_a_death() {
    the_chain_passes("a_chapter_taken_again_without_a_death");
}

/// And answered with the stage from its own start, which for this game is chapter 1 — so what this says
/// over the case above is that the item asks before it acts and that the confirmation is answered.
#[test]
fn the_stage_of_th06nc_taken_again_from_its_start() {
    the_chain_passes("the_stage_taken_again_from_its_start");
}

/// And answered with the run given up, which leaves the run for the front end.
#[test]
fn the_run_of_th06nc_given_up_at_the_retry() {
    the_chain_passes("the_run_given_up_at_the_retry");
}

/// **And a chapter further back**, which wants two of the fight's boundaries behind the one the run is
/// in: the question's second item lists the chapters behind, and the run plays back up to where it was.
#[test]
fn a_chapter_of_th06nc_further_back_taken_again() {
    the_chain_passes("a_chapter_further_back_taken_again");
}

/// And the same list over a death, the question the hit raised answered by the recipe and not the rules.
#[test]
fn a_death_of_th06nc_answered_by_a_chapter_further_back() {
    the_chain_passes("a_death_answered_by_a_chapter_further_back");
}

/// **The run left in its second chapter and picked up there** — the chain a stage's own boundary is
/// needed for, which this game has once its fight starts: the recipes wait on the boss before asking for
/// a second chapter, and both checks are th06c's unchanged.
#[test]
fn a_run_of_th06nc_left_in_its_second_chapter() {
    the_chain_passes("a_run_left_in_its_second_chapter");
}

/// **The stage after the first, and the run left there picked up in it** — one chain of two cases, the
/// second continuing in the directory the first left. What each says over the cases above is that a stage
/// a run *walked into* is a stage with a chapter of its own and a stage a pick-up can point a build at.
#[test]
fn a_chapter_of_th06nc_in_the_stage_after_the_first() {
    the_chain_passes("a_chapter_of_the_stage_after_the_first");
}

/// **The attempt a retry inside a spell card counts, and the run after it keeping that count** — the one
/// case here that plays two runs in a single launch, the put-back being spent at the build of a run's
/// first stage and nowhere else.
#[test]
fn an_attempt_at_a_card_of_th06nc_is_counted_and_kept() {
    the_chain_passes("an_attempt_at_a_card_counted_and_kept");
}

/// **And the mode question's other answer**: a run of the normal mode, which has no chapters, nothing
/// written down for it and no question of orb's over it. The one case of this game that asks nothing of a
/// boundary inside a stage and is therefore here rather than in the change's own TODO.
#[test]
fn a_normal_mode_run_of_th06nc_is_kept_nothing_of() {
    the_chain_passes("a_normal_mode_run_nothing_is_kept_of");
}

/// **One stage practised on its own, rewound like any other run and written down for like none** — the
/// main menu's item 2, which is where this game keeps a practice where th06, th07 and th06c each keep a
/// select of their own. What it says over every case above it: orb's mode question stands over that item
/// too, the run that starts reads as a practice, its chapter is taken again all the same, and nothing of
/// it is looked for on the disk or left there, a run of this kind having no slot to be kept in.
#[test]
fn a_stage_of_th06nc_practised_on_its_own() {
    the_chain_passes("a_stage_practised_on_its_own");
}

/// **And the whole game played to its clear, with the Extra that clear unlocks continuing it** — the
/// chain that costs the real game an hour and this one a minute: six stages with no bomb pressed, the
/// ending, the name typed at the entry it opens in, and then a run of the Extra's own stage on the
/// Extra's own difficulty whose boss leaves nearly out of life.
///
/// **What the two cases ask of the laid-out game is written in this file's own doc**: the Extra's stage,
/// its end being the screen a run ends at rather than an ending, and the life a beaten boss leaves with.
#[test]
fn a_run_of_th06nc_played_to_its_clear_and_the_extra_it_unlocked() {
    the_chain_passes("a_run_played_to_its_clear_without_a_bomb");
}

/// **And the same six stages in 完全無欠モード, with a chance of dying in each chapter** — which is the
/// mode's own whole-game case, where the chain above is the レガシーモード one the Extra is unlocked by.
///
/// **What the laid-out game says about it is that the recipe runs out and the check reads the trace**: a
/// chance of dying per chapter is the rules' and the fake's field is empty here — see [`WITHOUT_BULLETS`]
/// — so no death lands, and what this asserts is the walk, the ending and the name. The deaths are the
/// real game's to produce, and `checks::a_run_of_th06nc_played_to_its_clear` is what refuses a launch
/// that reached the ending without one.
#[test]
fn a_run_of_th06nc_played_to_its_clear_in_pointdevice_mode() {
    the_chain_passes("a_run_played_to_its_clear");
}

/// **A death in the nonspell after a card, answered by that nonspell again** — the laid-out midboss's own,
/// whose card at `CARD_STARTS` is the run's first and whose attack after it at `ATTACK_CHANGES` is the one
/// the hit lands in.
#[test]
fn a_death_in_the_nonspell_after_a_card_of_th06nc_answered_by_that_nonspell() {
    the_chain_passes("a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell");
}

/// **And the Normal run with no hit and no bomb**, over a field with nothing on it here — see
/// [`WITHOUT_BULLETS`]: what the laid-out game says of it is the walk, the ending, the name and the word
/// never pressing the bomb, the hits being the real game's to take.
#[test]
fn a_normal_run_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb() {
    the_chain_passes("a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb");
}
