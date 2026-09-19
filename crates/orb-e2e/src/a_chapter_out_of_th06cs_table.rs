//! A chapter of th06c that begins where the **baked** midstage table says, rather than where a fight does.
//!
//! A stage's waves are a script on a clock, so the boundaries between them are frame numbers somebody chose
//! and `game::th06c::chapters::MIDSTAGE` is where they are written down — th06 1.02h's rows, carried across
//! to its port. A run reads that table; a `--collect` or `--judge` pass reads the list it is building
//! instead, and that path is `a_chapter_table_collected_for_th06c.rs`. The two are different arms of
//! `Chapters::due`, and this is the one every played run takes.
//!
//! **The clock the table is written in is the script's**, `enemy_manager::TIMELINE_TIME`: a frame number
//! there lands on the same point of the wave pattern however fast the run is going and whatever difficulty it
//! is. A laid-out stage runs both of its clocks a frame at a time from nothing, so the frame the chapter
//! begins on is the script frame the table names.
//!
//! **The fight comes first**, which is what makes these runs longer than th07's: this laid-out game scripts a
//! midboss into every stage it builds — [`MIDBOSS_ARRIVES`] to [`MIDBOSS_BEATEN`] — and a fight underway
//! outranks the table, so the chapter the table's boundary begins is counted after the fight's own. What
//! th07's laid-out stage has in its place is nothing, its fights being driven from the test.
//!
//! The frame is not written here: it comes out of the table, so a stage tuned again is an e2e test that
//! follows it rather than one that has to be edited with it.

use crate::fake::th06c::{DIFFICULTY, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06c::chapters::MIDSTAGE;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The first stage, counted from zero as everything above `Game` counts.
const THE_FIRST_STAGE: i32 = 0;

/// And the Extra: the seventh and last row of the table. With the difficulty the game's own screens count
/// fifth, which is what a run of the Extra is played on and what the game's own run start gives the Extra's
/// lives and bombs by.
const THE_EXTRA: i32 = 6;
const THE_EXTRAS_DIFFICULTY: i32 = 4;

/// How many chapters the run has had by the time the table's boundary is reached: the stage's own start, the
/// midboss arriving, the card it puts up, the attack after that card, and the waves handed back when it goes
/// down.
///
/// Counted, because what a fixed number says is that nothing else began a chapter in between — the frames
/// from the fight to the boundary are the table's alone.
const CHAPTERS_BEFORE_IT: u32 = 5;

/// The frame the table's first boundary for that stage falls on, out of the table itself.
///
/// # Panics
/// On a row with nothing in it: a stage the table has no boundary for is one this e2e test has nothing to wait
/// for. And on a first boundary that is not past the laid-out fight: one inside it is spent by the fight
/// rather than beginning a chapter, and one before it moves every chapter of the fight along by one — rows
/// this test cannot be asked about as it stands.
fn first_boundary(stage: i32) -> i32 {
    let frame = MIDSTAGE[stage as usize]
        .first()
        .unwrap_or_else(|| panic!("a boundary in stage {}'s row of the table", stage + 1))
        .frame;
    assert!(
        frame > MIDBOSS_BEATEN as i32,
        "stage {}'s first boundary, {frame}, is not past the laid-out fight, which runs from \
         {MIDBOSS_ARRIVES} to {MIDBOSS_BEATEN}",
        stage + 1,
    );
    frame
}

/// A run of the game on `difficulty` standing in `stage`, with nothing fired at the player: the frames to the
/// boundary are thousands, and a death anywhere in them is the retry menu standing where the table was to be
/// read.
///
/// The difficulty before the run's start, which is the order the front end has them in: the select writes
/// it and the run start reads it.
fn in_the_stage(name: &str, stage: i32, difficulty: i32) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.image().the_difficulty_is(difficulty);
    game.in_a_run_in_the_stage(stage);
    assert_eq!(
        game.state().stage,
        stage,
        "the stage being played is not the one the run was put in",
    );
    game
}

/// Runs `game` to the frame the table names for its stage, and asserts the chapter that began there is the
/// table's — by the frame, by the clock the table is written in, and by the cause the log names.
fn plays_to_the_boundary(game: &Fake, stage: i32) {
    let boundary = first_boundary(stage);
    let said = format!(
        "stage {} chapter {} at frame {boundary} (script {boundary}): tl {boundary}",
        stage + 1,
        CHAPTERS_BEFORE_IT + 1,
    );
    let log = game.log();
    game.frames_until(&said, boundary as u32 + 60, || log.said(&said));
    assert_eq!(
        game.state().script_frames,
        boundary,
        "the chapter the table's boundary began is not on the frame the table names",
    );
    // And the fight really was over first: the chapter before this one is the waves handed back, which is
    // what says the boundary was not reached with a fight standing between the two.
    let handed_back = format!("stage {} chapter {CHAPTERS_BEFORE_IT} at frame", stage + 1);
    assert!(
        log.lines()
            .iter()
            .any(|line| line.contains(&handed_back) && line.contains("the midboss was beaten")),
        "the fight was not over before the table's boundary:\n  {}",
        log.lines().join("\n  ")
    );
}

/// The boundary stage one's row holds begins a chapter in a run of the game.
#[test]
fn the_boundary_th06cs_table_holds_for_a_stage_begins_a_chapter() {
    in_its_own_process(|| {
        let game = in_the_stage("th06c-table-boundary", THE_FIRST_STAGE, DIFFICULTY);
        plays_to_the_boundary(&game, THE_FIRST_STAGE);
    });
}

/// And the Extra reads the row of its own, which is the last of the table's seven: it is a stage like any
/// other to everything above `Game`, and the stage number is the whole of what picks its row.
#[test]
fn the_extra_of_th06c_reads_the_row_the_table_has_for_it() {
    in_its_own_process(|| {
        let game = in_the_stage(
            "th06c-table-boundary-extra",
            THE_EXTRA,
            THE_EXTRAS_DIFFICULTY,
        );
        plays_to_the_boundary(&game, THE_EXTRA);
    });
}
