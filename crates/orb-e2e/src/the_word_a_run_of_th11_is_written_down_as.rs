//! **A run of th11 that was paused is put back at its chapter all the same.**
//!
//! The game's own pause stops the run: the pause object sets the game object's flag `0x10` as it opens
//! (0x42c684), and the game object's job answers the walk with 3 while that flag stands (0x42076e through
//! 0x420789), so the stage's clock does not move and nothing behind it on the chain runs — the replay's copy of the
//! word at priority `0xb` among it — see `Th11`'s `GAME_PAUSED`. What that has to not do is put a frame of buttons
//! into the file for every update the menu stood: a chapter whose record held two hundred frames of nothing would
//! put the run back two hundred frames of nothing later.
//!
//! **And the button that opens it is one this game hands over**, `menu` among `Th11::button_bit`'s — Esc, bit
//! `0x100` of the word gone down, which the pause object opens on once the game object is thirty frames in
//! (0x42bf53, 0x42bf77) — so the recipe presses it the way somebody playing does and presses it again to let the
//! run go: the same bit on the menu closes it on its resume (0x42c9f2), and twelve frames later the menu's own
//! resume takes the flag off (0x42d460, 0x42c885). `the_word_a_run_of_th10_is_written_down_as` is the same claim
//! over that game.

use crate::fake::th11::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

/// How long a stage of the laid-out game lasts here: past its midboss's arrival at frame 1200, whose first
/// attack is the stage's second chapter.
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
    // Firing nothing, a run's dying being none of this file's subject.
    game.fires_at_the_player(false);
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
fn a_run_of_th11_that_was_paused_is_put_back_at_its_chapter() {
    in_its_own_process(|| {
        // The run, a stretch of its first chapter, the menu put up and taken down again, and then the chapter
        // after it — which is the chapter the file is read back at.
        let recipe = format!(
            "{TO_THE_SHOT}{TO_A_RUN}\
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
        let game = launched("th11-paused", recipe);
        let log = game.log();

        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("stage 1 chapter 1 (stage start) at frame")
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
