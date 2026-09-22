//! **A run of th08 that was paused is put back at its chapter all the same.**
//!
//! The game's own pause stops the run: its run manager answers the walk with 3 while the pause stands, so
//! neither clock moves and the frames it is up on are one frame of the run as far as the record is concerned
//! — see `Th08`'s `G_PAUSED`. What that has to not do is put a frame of buttons into the file for every
//! update the menu stood: a chapter whose record held two hundred frames of nothing would put the run back two
//! hundred frames of nothing later.
//!
//! **And the button that opens it is one this game hands over**, `menu` among `Th08::button`'s, so the recipe
//! presses it the way somebody playing does and presses it again to let the run go — which is the menu's own
//! way back, its closing phase twenty updates long. `the_word_a_run_of_th06c_is_written_down_as` is the same
//! claim over that game.

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

/// How long a stage of the laid-out game lasts here: past stage 1's midstage table and into the boss's fight,
/// she arriving for its last 900 frames.
const A_STAGE: u32 = 4_500;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the recipe never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  ")
    );
}

#[test]
fn a_run_of_th08_that_was_paused_is_put_back_at_its_chapter() {
    in_its_own_process(|| {
        // The run, a stretch of its first chapter, the menu put up and taken down again, and then the chapter
        // after it — which is the chapter the file is read back at.
        let recipe = format!(
            "{TO_A_RUN}\
- play_until: {{ frames: 60 }}
- press: [menu]
  frames: 6
- wait_until: {{ frames: 120 }}
- press: [menu]
  frames: 6
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th08-paused", recipe);
        let log = game.log();

        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        // The menu up, which the game's own read of that button does: while it is up the stage's clock stands.
        game.frames_until("the game held", FRAMES, || game.state().paused);
        let stopped = game.state().stage_frames;
        for _ in 0..60 {
            game.frame();
        }
        assert_eq!(
            game.state().stage_frames,
            stopped,
            "the stage went on with the game's own menu up:\n  {}",
            log.lines().join("\n  ")
        );

        // And down again, the run going on from where it stood.
        game.frames_until("the game let go", FRAMES, || !game.state().paused);
        game.frames_until("the stage running again", FRAMES, || {
            game.state().stage_frames > stopped
        });

        // Then the chapter on the other side of it, whose record is the frames of the run and not the updates
        // the menu stood for — waited for by the line the *write* leaves, the write being a thread of orb's.
        let written_down = |log: &orb_sim::Log| {
            log.lines()
                .into_iter()
                .find(|line| line.contains("chapter 2") && line.contains("frame(s) of buttons"))
        };
        game.frames_until("the chapter past the pause written down", FRAMES, || {
            written_down(log).is_some()
        });
        let written = written_down(log).unwrap_or_else(|| {
            panic!(
                "the chapter past the pause was not written down:\n  {}",
                log.lines().join("\n  ")
            )
        });
        // The frames of buttons the file holds, against the stage's own clock at that chapter: the two are the
        // same count, which is what says the pause put none in.
        let buttons: u32 = written
            .split(" frame(s) of buttons")
            .next()
            .and_then(|before| before.rsplit(' ').next())
            .and_then(|it| it.parse().ok())
            .unwrap_or_else(|| {
                panic!("the line does not say how many frames of buttons: {written}")
            });
        let at: u32 = written
            .split(" at frame ")
            .nth(1)
            .and_then(|rest| rest.split(',').next())
            .and_then(|it| it.parse().ok())
            .unwrap_or_else(|| panic!("the line does not say the frame it began on: {written}"));
        assert_eq!(
            buttons, at,
            "the record holds a frame of buttons for frames the run did not have: {written}",
        );
        played_out(&game);
    });
}
