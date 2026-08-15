//! A practice run: one stage played on its own, which is a run orb rewinds and a run it writes nothing
//! down for.
//!
//! **Both halves are the same fact from two sides.** A practice run is a run — the mode question goes
//! over `Practice Start` like it goes over `Game Start`, and the chapters, the snapshots and the retry
//! menu are a stage's whether the stage was reached by playing to it or chosen from a menu. What it has
//! not got is a name: `Game::run_slot` answers `None`, so there is no file for the chapter it is in, and
//! nothing to offer the next time the same stage is practised. It is one stage started again from the
//! game's own menu in less time than a playback takes.
//!
//! The Extra is the other run that is not a full one, and it is the opposite answer: a name of its own.
//! That one is `a_chapter_out_of_the_table.rs`.

use crate::fake::th06::{CARD, CARD_STARTS, Fake};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::RunStart;
use orb_core::game::th06::image::{Screen, item};
use orb_sim::keys;

/// The stage this practises, counted from zero the way everything above `Game` counts them: stage 3.
///
/// Its row of the midstage table begins at script 1009, which is past every frame played here — so the
/// chapters below are the fight's own, and a boundary out of the table is
/// `a_chapter_out_of_the_table.rs`.
const THE_STAGE: i32 = 2;

/// The run: that stage on Normal with Reimu A, practised.
fn the_practice_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: true,
        stage: THE_STAGE,
    }
}

#[test]
fn a_practice_run_is_rewound_and_nothing_is_written_down_for_it() {
    in_its_own_process(|| {
        let game = Fake::attach("practice", the_practice_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.at_the_title_menu();

        // ── 1. The question, over `Practice Start`. Which item the cursor is on is what orb reads to know
        // there is anything to ask about, and all three of the items that start a run are runs: the
        // question is the same one, and answering it decides whether this stage can be rewound.
        for _ in 0..item::PRACTICE {
            game.press(keys::DOWN);
        }
        assert_eq!(
            game.image().front_end_now().cursor,
            item::PRACTICE,
            "the cursor is not on the item a practice stage is started from",
        );
        game.press(keys::Z);
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press over Practice Start asked nothing: {:?}",
            log.lines(),
        );
        game.press_until(keys::Z, "the mode chosen", || {
            log.said("mode: answered on the keyboard")
        });

        // ── 2. The stage the practice screen chose, which is the one the run is played in — and the run
        // says it is a practice one, which is the whole of what tells it from a full run that has reached
        // the same stage.
        //
        // Two screens rather than one: the shot type select sends a practice run to the stage select
        // instead of into the run, that being where the stage it is played in is answered for.
        game.frames_until("the shot type select", 90, || {
            let front = game.image().front_end_now();
            front.screen == Screen::ShotType && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames_until("the practice stage select", 90, || {
            let front = game.image().front_end_now();
            front.screen == Screen::PracticeStage && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames_until("the stage built", 8, || {
            let state = game.state();
            state.playing && state.stage_frames >= 1
        });
        let state = game.state();
        assert!(state.practice, "the run reads as a full run");
        assert_eq!(state.stage, THE_STAGE);

        // ── 3. Its chapters are a stage's like any other's: the stage's own start, and the card the fight
        // puts up.
        game.frames_until("the stage's first chapter", 400, || {
            log.said("stage 3 chapter 1 (stage start)")
        });
        game.frames_until("the card's chapter", CARD_STARTS + 60, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });
        let at_the_card = game.state();

        // ── 4. 被弾, and the chapter put back: what the mode promises is the memory as it was, which for a
        // practice run is the same mechanism and the same snapshot as for a full one.
        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter 3"),
            "the death was not noticed: {:?}",
            log.lines(),
        );
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the practice run is not where the chapter began, field for field",
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the attempt the retry is was not counted against the card",
        );

        // ── 5. And nothing of it on the disk. A run with no slot is not looked for and not written down:
        // there was no question about where to start it — which is what says the file was never even
        // asked about — and the chapters it has reached since have left nothing behind.
        assert!(
            !log.said("resume: nothing was left of"),
            "a practice run was looked up as a run that could have been left: {:?}",
            log.lines(),
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a practice run left a chapter behind: {:?}",
            orb_core::resume::left(game.dir()),
        );
    });
}
