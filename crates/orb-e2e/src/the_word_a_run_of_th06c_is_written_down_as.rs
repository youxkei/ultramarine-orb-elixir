//! **A run of th06c that was paused is put back at its chapter all the same.**
//!
//! The game's own pause stops the run: neither clock moves while the menu is up, so the frames it is open on
//! are one frame of the run as far as the record is concerned. What that has to not do is put a frame of
//! buttons into the file for every frame the menu stood — a chapter whose record held two hundred frames of
//! nothing would put the run back two hundred frames of nothing later.
//!
//! **And the button that opens it is one this game hands over**, `menu` among `Th06c::BUTTONS`, so the recipe
//! presses it the way somebody playing does rather than the image being told the game is held. Which is what
//! makes this about the word: the pause goes in through the read orb feeds, and the run comes back from a
//! chapter taken on the other side of it.
//!
//! `the_word_a_run_of_th07_is_written_down_as` is the same claim over the game that keeps two words and copies
//! one into the other; th06c keeps one — see `th06c/asm.md`.

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

#[test]
fn a_run_of_th06c_that_was_paused_is_put_back_at_its_chapter() {
    in_its_own_process(|| {
        // The run, its first chapter, the menu put up and taken down again, and then the chapter after it —
        // which is the chapter the file is read back at.
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ frames: 60 }}
- press: [menu]
  frames: 6
- wait_until: {{ frames: 120 }}
- press: [decide]
  frames: 6
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th06c-paused", recipe);
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

        // Then the chapter on the other side of it, whose record is the frames of the run and not the frames
        // the menu stood for.
        game.frames_until("the chapter past the pause", FRAMES, || {
            log.said("stage 1 chapter 2")
        });
        let written = log
            .lines()
            .into_iter()
            .find(|line| line.contains("chapter 2") && line.contains("frame(s) of buttons"))
            .unwrap_or_else(|| {
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
