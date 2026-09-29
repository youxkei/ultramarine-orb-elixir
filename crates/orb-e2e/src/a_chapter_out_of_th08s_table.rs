//! A chapter of th08 that begins where the **baked** midstage table says, rather than where a fight does.
//!
//! What `a_chapter_out_of_th06cs_table.rs` asks of that game, asked of this one's nine rows —
//! `game::th08::chapters::MIDSTAGE`, chosen by th06's and th07's rules out of what a `--collect` pass
//! proposed. A run reads that table; a `--collect` pass reads the list it is building instead, and that path
//! is `a_chapter_table_collected_for_th08.rs`.
//!
//! **The clock the table is written in is the script's**, `State::script_frames`, which the real game starts
//! again with every stage while the run's own clock carries on — see `Th08`'s `SCRIPT_CONTEXTS`, where a
//! launch read `stage=1 frames=17132 script=0` on the first frame of stage 2. So a row of any stage but the
//! first is a row read against a clock that stage began at nothing, and that is what the stages after the
//! first are here to say.
//!
//! **And the row is picked by the stage word, counted from nothing over nine**: the halves the fourth stage
//! and the last are split into, and the Extra, are rows of their own. A run of the laid-out game is put in
//! one of them by that word and nothing else, so a row read off the wrong index is a chapter on a frame
//! another stage's row names.
//!
//! **No fight comes first**: this laid-out stage has no midboss and its boss arrives for the stage's last
//! 900 frames — see `fake::th08`'s `THE_BOSS_FIGHTS_FOR` — so a stage that lasts past the table's last row
//! for it is its own start and then the table's rows, one chapter each.
//!
//! The frames are not written here: they come out of the table, so a stage tuned again is an e2e test that
//! follows it rather than one that has to be edited with it.

use crate::fake::th08::{DIFFICULTY_NORMAL, Fake, THE_EXTRAS_DIFFICULTY};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th08::chapters::MIDSTAGE;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage lasts here: past the last row of every stage's table and a boss arriving 900 frames
/// before its end, so that nothing but the table begins a chapter over what this plays.
const A_STAGE: u32 = 20_000;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The nine stage words, which is what picks a row of the table.
const STAGE_1: i32 = 0;
const STAGE_2: i32 = 1;
const STAGE_3: i32 = 2;
const THE_UNCANNY_FOURTH: i32 = 3;
const THE_POWERFUL_FOURTH: i32 = 4;
const STAGE_5: i32 = 5;
const EIRINS_STAGE: i32 = 6;
const KAGUYAS_STAGE: i32 = 7;
const THE_EXTRA: i32 = 8;

/// **A run of the game on `difficulty`, walked into stage word `stage`** — placed in the first and then the
/// run built again at that one, which is how this game moves a run on — played by the stage's own script
/// with nothing fired at the player: the frames to the last row are thousands, and a death anywhere in them
/// is the retry menu standing where the table was to be read.
fn in_the_stage(name: &str, stage: i32, difficulty: i32) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.stages_last(A_STAGE);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.image().is_playing(STAGE_1, difficulty);
    game.plays_its_own_stage();
    // **Played in the first stage for a while before it is left**, so that the stage walked into begins its
    // script clock with the run's clock well past nought: a row read against the run's clock, or against a
    // script clock the build did not start again, is then a row read on the wrong frame.
    game.frames(A_WHILE_IN_THE_FIRST);
    game.moves_on_to_the_stage(stage);
    let log = game.log();
    let said = format!("stage {} chapter 1 (stage start)", stage + 1);
    game.frames_until("the stage's own first chapter", FRAMES_FOR_A_BUILD, || {
        game.state().stage == stage && log.said(&said)
    });
    game
}

/// How long the run is in the first stage before it is moved on: short of that stage's own row, so the
/// first stage's chapters are its start and nothing else.
const A_WHILE_IN_THE_FIRST: u32 = 600;

/// Runs `game` through every row the table has for `stage`, and asserts each began a chapter of its own — by
/// the chapter's number, by the clock the table is written in, and by the cause the log names.
fn plays_through_the_rows(game: &Fake, stage: i32) {
    let rows = MIDSTAGE[stage as usize];
    assert!(
        !rows.is_empty(),
        "stage word {stage} has no row in the table to be read",
    );
    let log = game.log();
    for (at, row) in rows.iter().enumerate() {
        let said = format!("stage {} chapter {} at frame", stage + 1, at + 2,);
        game.frames_until(&said, row.frame as u32 + 120, || log.said(&said));
        let line = log
            .lines()
            .into_iter()
            .find(|line| line.contains(&said))
            .expect("the line just waited for");
        assert!(
            line.contains(&format!("(script {}): tl {}", row.frame, row.frame)),
            "the chapter began on stage word {stage}'s row {at} is not on script frame {}:\n  {line}",
            row.frame,
        );
        assert_eq!(
            game.state().script_frames,
            row.frame,
            "the chapter the table's row began is not on the frame the table names",
        );
    }
    // And nothing else began one in between: the frames from the stage's start to its last row are the
    // table's alone.
    let beyond = format!("stage {} chapter {} at frame", stage + 1, rows.len() + 2);
    assert!(
        !log.said(&beyond),
        "a chapter the table has no row for began in stage word {stage}:\n  {}",
        log.lines().join("\n  ")
    );
}

fn the_rows_of(stage: i32, difficulty: i32) {
    in_its_own_process(|| {
        let game = in_the_stage(
            &format!("th08-table-rows-of-stage-word-{stage}"),
            stage,
            difficulty,
        );
        plays_through_the_rows(&game, stage);
    });
}

/// The row stage 1 holds begins a chapter in a run of the game — the one stage whose script clock and run
/// clock are the same number, placed there rather than walked to.
#[test]
fn the_row_th08s_table_holds_for_its_first_stage_begins_a_chapter() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-table-rows-of-the-first-stage",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.plays_its_own_stage();
        let log = game.log();
        game.frames_until("the stage's own first chapter", FRAMES_FOR_A_BUILD, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        plays_through_the_rows(&game, STAGE_1);
    });
}

#[test]
fn the_rows_th08s_table_holds_for_stage_2_begin_chapters_on_its_own_script_clock() {
    the_rows_of(STAGE_2, DIFFICULTY_NORMAL);
}

#[test]
fn the_row_th08s_table_holds_for_stage_3_begins_a_chapter() {
    the_rows_of(STAGE_3, DIFFICULTY_NORMAL);
}

/// The fourth stage is two rows of the table, which only the stage word tells apart.
#[test]
fn the_uncanny_fourth_stage_of_th08_reads_the_row_of_its_own() {
    the_rows_of(THE_UNCANNY_FOURTH, DIFFICULTY_NORMAL);
}

#[test]
fn the_powerful_fourth_stage_of_th08_reads_the_row_of_its_own() {
    the_rows_of(THE_POWERFUL_FOURTH, DIFFICULTY_NORMAL);
}

#[test]
fn the_rows_th08s_table_holds_for_stage_5_begin_chapters() {
    the_rows_of(STAGE_5, DIFFICULTY_NORMAL);
}

/// And so is the last.
#[test]
fn eirins_stage_of_th08_reads_the_row_of_its_own() {
    the_rows_of(EIRINS_STAGE, DIFFICULTY_NORMAL);
}

#[test]
fn kaguyas_stage_of_th08_reads_the_row_of_its_own() {
    the_rows_of(KAGUYAS_STAGE, DIFFICULTY_NORMAL);
}

/// And the Extra reads the ninth and last row, on the difficulty the game's own screens count fifth: it is a
/// stage like any other to everything above `Game`, and the stage word is the whole of what picks its row.
#[test]
fn the_extra_of_th08_reads_the_row_the_table_has_for_it() {
    the_rows_of(THE_EXTRA, THE_EXTRAS_DIFFICULTY);
}
