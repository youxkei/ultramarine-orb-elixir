//! A chapter of th11 that begins where the **baked** midstage table says, rather than where a fight does.
//!
//! What `a_chapter_out_of_th10s_table.rs` asks of that game, asked of this one's rows —
//! `game::th11::chapters::MIDSTAGE`, chosen by th08's rules out of what a `--tune` pass proposed. A run reads that
//! table; a `--collect` pass reads the list it is building instead.
//!
//! **The clock the table is written in is the script's**, `State::script_frames`: the frames of the stage none of
//! its own scripts was held on (`Th11::script_frames`). Nothing of a laid-out stage's script is played here, so
//! nothing holds it and it is the stage's own frames, which count again from nought in a stage walked into — which
//! walking the run into that stage is here to say.
//!
//! **And the row is picked by the stage word**, counted from nothing over seven: stages 3 and 6 have no row, so the
//! stages here are 1, 2, 4, 5 and the Extra, which a run of its own is placed in rather than walked to.
//!
//! **No fight comes first**: these stages are the laid-out game's with nothing of its own script played — no
//! midboss, no boss, no fire — so a stage is its own start and then the table's rows, one chapter each.
//!
//! The frames are not written here: they come out of the table, so a stage tuned again is an e2e test that follows
//! it rather than one that has to be edited with it.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th11::chapters::MIDSTAGE;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a stage's build, its fade and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 600;

/// The stage words that hold rows, counted from nought.
const STAGE_1: i32 = 0;
const STAGE_2: i32 = 1;
const STAGE_4: i32 = 3;
const STAGE_5: i32 = 4;
const EXTRA: i32 = 6;

/// How long the run is in each stage before it is moved on: short of any row, so a stage walked through has its
/// start and nothing else.
const A_WHILE_IN_A_STAGE: u32 = 300;

fn launched(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game
}

/// **A run walked into stage word `stage`** — placed in the first, and moved on a stage at a time the way the
/// conversation after a boss moves it (0x41eda3), each through the stage after one won's build — with nothing of
/// the stage's own script played and nothing fired at the player.
fn in_the_stage(name: &str, stage: i32) -> Box<Fake> {
    let game = launched(name);
    game.is_in_a_run();
    let log = game.log();
    for walked_into in 0..=stage {
        let said = format!("stage {} chapter 1 (stage start)", walked_into + 1);
        game.frames_until("the stage's own first chapter", FRAMES_FOR_A_BUILD, || {
            game.state().stage == walked_into && log.said(&said)
        });
        if walked_into < stage {
            game.frames(A_WHILE_IN_A_STAGE);
            game.moves_on_to_the_next_stage();
        }
    }
    game
}

/// Runs `game` through every row the table has for `stage`, and asserts each began a chapter of its own — by the
/// chapter's number, by the clock the table is written in, and by what the log says of it.
fn plays_through_the_rows(game: &Fake, stage: i32) {
    let rows = MIDSTAGE[stage as usize];
    assert!(
        !rows.is_empty(),
        "stage word {stage} has no row in the table to be read",
    );
    let log = game.log();
    for (at, row) in rows.iter().enumerate() {
        let said = format!("stage {} chapter {} at frame", stage + 1, at + 2);
        game.frames_until(&said, row.frame as u32 + 120, || log.said(&said));
        let line = log
            .lines()
            .into_iter()
            .find(|line| line.contains(&said))
            .expect("the line just waited for");
        assert!(
            line.contains(&format!("(script {})", row.frame)),
            "the chapter begun on stage word {stage}'s row {at} is not on script frame {}:\n  {line}",
            row.frame,
        );
        assert_eq!(
            game.state().script_frames,
            row.frame,
            "the chapter the table's row began is not on the frame the table names",
        );
    }
    // And nothing else began one in between: the frames from the stage's start to its last row are the table's
    // alone.
    let beyond = format!("stage {} chapter {} at frame", stage + 1, rows.len() + 2);
    assert!(
        !log.said(&beyond),
        "a chapter the table has no row for began in stage word {stage}:\n  {}",
        log.lines().join("\n  ")
    );
}

fn the_rows_of(stage: i32) {
    in_its_own_process(|| {
        let game = in_the_stage(&format!("th11-table-rows-of-stage-word-{stage}"), stage);
        plays_through_the_rows(&game, stage);
    });
}

#[test]
fn the_row_th11s_table_holds_for_stage_1_begins_a_chapter_on_its_own_script_clock() {
    the_rows_of(STAGE_1);
}

#[test]
fn the_rows_th11s_table_holds_for_stage_2_begin_chapters() {
    the_rows_of(STAGE_2);
}

#[test]
fn the_rows_th11s_table_holds_for_stage_4_begin_chapters() {
    the_rows_of(STAGE_4);
}

#[test]
fn the_rows_th11s_table_holds_for_stage_5_begin_chapters() {
    the_rows_of(STAGE_5);
}

/// **And the Extra's**, a run placed in it the way its item of the main menu starts one.
#[test]
fn the_rows_th11s_table_holds_for_the_extra_begin_chapters() {
    in_its_own_process(|| {
        let game = launched("th11-table-rows-of-the-extra");
        game.is_in_the_extra();
        let log = game.log();
        game.frames_until("the Extra's own first chapter", FRAMES_FOR_A_BUILD, || {
            game.state().stage == EXTRA && log.said("stage 7 chapter 1 (stage start)")
        });
        plays_through_the_rows(&game, EXTRA);
    });
}
