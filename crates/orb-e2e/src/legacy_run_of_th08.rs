//! **One legacy mode run of th08, which is the same game with none of orb's own work happening in it.**
//!
//! `a_pointdevice_run_of_th08.rs` is what the mode *does*; this is what answering the other way costs, and it
//! is worth an e2e test of its own because every one of those things is something orb has to **not** do. A run
//! whose chapters were kept anyway would look exactly like this one until somebody died — which is the one
//! thing the real-game case `a_normal_mode_run_nothing_is_kept_of` does not ask, the rules there dodging what
//! they can.
//!
//! **Walked into by the real recipes' own walk with the other answer**: orb's question over the title answered
//! with its second item, レガシーモード, and the run the game's own team select then starts.

use orb_config::LogLevel;

use crate::fake::th08::{BOMBS, Fake, LIVES};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage lasts here, which is longer than this test plays: what is watched is a death inside one
/// stage, so the stage must not end under it.
const A_STAGE: u32 = 100_000;

/// The most frames the walk is given to reach its last step.
const FRAMES_FOR_THE_WALK: u32 = 6_000;

/// How far past the stage's start the run is played before the hit: past the frame Pointdevice mode takes the
/// stage's first chapter on, which is what makes its absence something this can read.
const PAST_THE_FIRST_CHAPTER: u32 = 600;

/// **`fake::th08::TO_A_RUN` with the mode question answered the other way**: its second item, レガシーモード.
const TO_A_LEGACY_RUN: &str = "
- wait_until: { screen: 0 }
  give_up_after: 1800
- press: [up]
  frames: 30
  until: { cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 1 }
  give_up_after: 300
- wait_until: { screen: 4 }
  give_up_after: 1800
- press: [decide]
  frames: 30
  until: { screen: 5 }
  give_up_after: 900
- press: [decide]
  frames: 12
- wait_until: { frames: 120 }
- press: [decide]
  frames: 12
- wait_until: { frames: 300 }
";

#[test]
fn a_legacy_run_of_th08_keeps_no_chapters_offers_no_retry_and_leaves_nothing_behind() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-legacy",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(TO_A_LEGACY_RUN.to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.plays_its_own_stage();
        game.stages_last(A_STAGE);

        // ── 1. Legacy mode, and the stage it starts. The one thing orb does to the run is write down that it
        // is doing nothing.
        game.frames_until("the walk into a run", FRAMES_FOR_THE_WALK, || {
            game.state().in_run && game.state().stage_frames > PAST_THE_FIRST_CHAPTER
        });
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the walk did not choose レガシーモード:\n  {}",
            log.lines().join("\n  ")
        );
        game.one_frame_to_drain_the_log();
        assert!(
            log.said("resume: stage 1 of a run orb is not keeping; nothing of it is written down"),
            "the run was written down after all:\n  {}",
            log.lines().join("\n  ")
        );

        // ── 2. The stage played past where Pointdevice mode's own first chapter was, and it is not here: no
        // snapshot is taken, so there is nothing to go back to.
        assert!(
            !log.said("chapter 1 (stage start)"),
            "a chapter was taken in a run that cannot be rewound:\n  {}",
            log.lines().join("\n  ")
        );

        // ── 3. The hit. A life, and no menu: the game takes the death and the run carries on.
        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.one_frame();
        let died_at = game.state().stage_frames;
        game.frames(10);
        assert!(
            !log.said("died in chapter"),
            "the retry menu was offered in a run with nothing to retry:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.state().stage_frames > died_at,
            "the game stopped updating, which is what a retry menu would do to it",
        );
        // And the life it lost is the run's own: a mode that keeps nothing takes it for good.
        assert_eq!(
            (game.state().lives, game.state().deaths),
            (LIVES as i8 - 1, 1),
            "the death was put back in a run that keeps nothing",
        );

        // ── 4. And nothing is left behind for a later launch to offer.
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a run that keeps nothing left a chapter behind: {:?}",
            orb_core::resume::left(game.dir()),
        );
    });
}
