//! **A clear of th11's own mode kept through orb's mode question for the run after it** — th10's claim, held to this
//! game's own score object.
//!
//! The game writes its score object back to `scoreth11.dat` only from the game object's teardown and as the game
//! leaves (0x437a70 at 0x41fe17, 0x428d81), and the ending marks the clear in the object after the run has been torn
//! down on its way there (0x40ede8 through 0x40edfa). So until the next teardown the clear is in memory alone — and
//! orb's mode question over the next run can read the mode's file into the object
//! (`Game::read_what_the_modes_file_records`). Where the object already holds that file's records it has to leave
//! what the game set in memory since its last write where it is, or the next teardown writes the clear away — and
//! with it the Extra the main menu lights out of it (0x439350).
//!
//! **Played by the real case's own recipe for the clear**, with the mode question for the next run asked and
//! answered with the game's own mode in place of the game's own way out, and the clear read back out of the score
//! object.

use orb_config::LogLevel;
use orb_core::game::th11::score_file;

use crate::fake::th11::{Fake, THE_GAMES_SCORE_FILE};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage of the laid-out game lasts here, as the real cases are played over it.
const A_STAGE: u32 = 4_500;

/// How many frames the launch is given to play the whole run and the question after it.
const FRAMES: u32 = 120_000;

/// **Where the ending of Reimu with her first partner and no continue is in the score object's settings**: the
/// endings seen from `+0x2ddea`, the ending `character * 3 + partner` on by six (0x40edb5 through 0x40edc0), the
/// settings starting at `+0x2ddd4` — and lit where its bit `0x10` stands, which a run on Normal or above writes.
const REIMU_AND_YUKARIS_CLEAR: usize = 0x2_ddea + 6 - 0x2_ddd4;
const A_CLEAR_ON_NORMAL: u8 = 0x10;

/// **The mode question for the next run, answered with the game's own mode** — asked over the main menu's first
/// item again, and the game closed on the difficulty select it leads to.
const THE_NEXT_RUNS_QUESTION: &str = "\
- press: [up]
  frames: 10
  until: { screen: 1, cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { question: mode }
  give_up_after: 900
- answer: { question: mode, item: 1 }
  give_up_after: 300
- wait_until: { screen: 5 }
  give_up_after: 1800
- quit: true
";

/// The real case's recipe with [`THE_NEXT_RUNS_QUESTION`] in place of its way out.
fn the_clear_and_the_next_runs_question() -> String {
    let recipe =
        include_str!("../../orb-real/recipes/th11/a_normal_mode_run_played_to_its_clear.yaml");
    let way_out = recipe
        .find("- press: [cancel]\n  frames: 10\n  until: { screen: 1, cursor: 7 }")
        .expect("the recipe leaves through the main menu's last item");
    format!("{}{THE_NEXT_RUNS_QUESTION}", &recipe[..way_out])
}

/// **The clear is in the score object the next teardown writes**, a mode question answered with the game's own
/// mode after the clear notwithstanding.
#[test]
fn a_clear_of_th11_is_kept_through_the_mode_question_of_the_run_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-clear-kept",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(the_clear_and_the_next_runs_question());
                config.log_level = LogLevel::Verbose;
            },
        );
        // A score file to start from, as every copy of the game has: a game that finds none writes none.
        let mut records = score_file::Records::default();
        records.shot_types[0] = Some(score_file::shot_types_record(0, 0));
        game.sim()
            .files()
            .put(THE_GAMES_SCORE_FILE, score_file::encode(&records));
        game.plays_its_own_stage();
        game.stages_last(A_STAGE);
        for _ in 0..FRAMES {
            if game.sim().exited().is_some() {
                break;
            }
            game.frame();
        }
        let log = game.log();
        assert!(
            game.sim().exited().is_some(),
            "the launch never left:\n  {}",
            log.lines()[log.lines().len().saturating_sub(60)..].join("\n  ")
        );
        assert!(
            log.said("mode: normal, was normal"),
            "no mode question was answered after the clear, so this says nothing:\n  {}",
            log.lines().join("\n  ")
        );
        assert_ne!(
            game.image().the_score_objects_settings()[REIMU_AND_YUKARIS_CLEAR] & A_CLEAR_ON_NORMAL,
            0,
            "the clear was taken out of the score object by the question after it:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| {
                    ["score:", "mode:", "autoplay: step", "recipe"]
                        .iter()
                        .any(|said| line.contains(said))
                })
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    });
}
