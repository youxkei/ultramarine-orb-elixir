//! A chapter of th07 that begins where the **baked** midstage table says, rather than where a fight does.
//!
//! A stage's waves are a script on a clock, so the boundaries between them are frame numbers somebody chose
//! and `game::th07::chapters::MIDSTAGE` is where they are written down — every one of them a hand's, this
//! game having no gaps for the detector to propose into. A run reads that table; a `--collect` or `--judge`
//! pass reads the list it is building instead, and that path is `a_chapter_table_collected_for_th07.rs`.
//! The two are different arms of `Chapters::due`, and this is the one every played run takes.
//!
//! **The clock the table is written in is the script's**: a frame number there lands on the same point of the wave pattern however fast the run is going and
//! whatever difficulty it is. A laid-out stage runs both of its clocks a frame at a time from nothing, so the
//! frame the chapter begins on is the script frame the table names.
//!
//! The frame is not written here: it comes out of the table, so a stage tuned again is an e2e test that
//! follows it rather than one that has to be edited with it.

use crate::fake::th07::{FIRST_STAGE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::chapters::MIDSTAGE;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frame the table's first boundary for that stage falls on, out of the table itself.
///
/// # Panics
/// On a row with nothing in it: a stage the table has no boundary for is one this e2e test has nothing to wait
/// for.
fn first_boundary(stage: i32) -> i32 {
    MIDSTAGE[stage as usize]
        .first()
        .unwrap_or_else(|| panic!("a boundary in stage {}'s row of the table", stage + 1))
        .frame
}

/// The boundary stage one's row holds begins a chapter in a run of the game — the second of the stage, its own
/// start being the first and nothing of a fight standing between the two.
#[test]
fn the_boundary_th07s_table_holds_for_a_stage_begins_a_chapter() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-table-boundary",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        let boundary = first_boundary(FIRST_STAGE);
        let said =
            format!("stage 1 chapter 2 at frame {boundary} (script {boundary}): tl {boundary}");
        game.frames_until(&said, boundary as u32 + 60, || log.said(&said));
        assert_eq!(
            game.state().script_frames,
            boundary,
            "the chapter the table's boundary began is not on the frame the table names",
        );
        // And by the table rather than by anything else that begins a chapter: no fight was fought, so a
        // chapter between the stage's start and this one would be one nothing here asked for.
        assert!(
            !log.said("stage 1 chapter 3"),
            "a chapter began that neither the stage's start nor the table accounts for:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
