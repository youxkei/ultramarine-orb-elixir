//! **The chapter a hit keeps where a run of th11 plays at four updates a drawn frame**, which is how the real cases
//! play a whole game.
//!
//! What `--keep-hits-in` keeps is a copy of the chapter written down at that chapter's start, and a chapter is
//! written down only on the frame it began on — so a run that stood on one frame in four passed every chapter whose
//! first frame was not one it drew, and the file the copy was taken of was an earlier chapter's. Kept under the later
//! chapter's name, it was a run that came back where the name did not say: a chapter kept off a real case as
//! `s7-c13-f38696` put a launch back at the Extra's first chapter (2026-10-08). And a retry goes back to the same
//! file: on real4 a hit in chapter 10 was taken again from chapter 7.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A stage whose boss comes up on an update the run does not draw: the laid-out boss's first attack begins `900`
/// before the stage ends, at frame `2,701`, a frame past a multiple of four.
const A_STAGE: u32 = 3_601;
const THE_BOSS_COMES_AT: u32 = A_STAGE - 900;

/// How many frames a screen or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// **How many updates each drawn frame runs**, the speed the real cases play a whole game at.
const UPDATES_A_FRAME: u32 = 4;

const HITS: &str = r"C:\kept\hits";

/// **Into a Pointdevice run and played**, by a recipe: the speed is a recipe's, a launch with none playing at one
/// update a frame whatever it is given.
const INTO_A_POINTDEVICE_RUN: &str = "\
- wait_until: { screen: 1 }
  give_up_after: 3600
- press: [decide]
  frames: 10
  until: { question: mode }
  give_up_after: 900
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 5 }
  give_up_after: 1800
- press: [decide]
  frames: 10
  until: { screen: 6 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { screen: 7 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { stage: 1, chapter: 0 }
  give_up_after: 900
- play_until: { frames: 400000 }
";

/// **A chapter that begins between two drawn frames is drawn on its first frame and written down**, and a hit in it
/// keeps it under its own name.
#[test]
fn a_chapter_of_th11_begun_between_drawn_frames_at_four_updates_a_frame_is_written_down_and_kept_for_its_hit()
 {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-hit-kept-at-four-updates-a-frame",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(INTO_A_POINTDEVICE_RUN.to_owned());
                config.log_level = LogLevel::Verbose;
                config.speed = UPDATES_A_FRAME;
                config.keep_hits_in = Some(HITS.into());
            },
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        game.frames_until("the run", FRAMES_FOR_A_SCREEN * 4, || game.state().in_game);

        // The boss's first attack, begun on an update past a multiple of four.
        let begun = format!("at frame {THE_BOSS_COMES_AT} ");
        game.frames_until("the boss's first attack", A_STAGE, || {
            log.lines()
                .iter()
                .any(|line| line.contains(" chapter ") && line.contains(&begun))
        });
        let chapter = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("stage 1 chapter ")?;
                let (number, rest) = after.split_once(' ')?;
                rest.starts_with(&begun).then(|| number.to_owned())
            })
            .expect("the boss's attack began no chapter");
        game.frames_until("the chapter written down", 4, || {
            log.said(&format!("resume: stage 1 chapter {chapter} "))
        });

        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || {
            log.said(&format!("died in chapter {chapter}"))
        });
        assert!(
            log.said("resume: the chapter hit in kept as"),
            "the chapter hit in was not kept:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("not written down at its start; the hit in it not kept"),
            "the hit's chapter was said to be unwritten:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
