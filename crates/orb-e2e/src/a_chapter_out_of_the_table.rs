//! A chapter that begins where the **baked** midstage table says, rather than where a fight does.
//!
//! A stage's waves are a script on a clock, so the boundaries between them are frame numbers somebody
//! chose and `game::th06::chapters::MIDSTAGE` is where they are written down. A run reads that table; a
//! `--collect` or `--judge` pass reads the list it is building instead, and that path is
//! `a_chapter_table_collected.rs`. The two are different arms of `Chapters::due`, and this is the one
//! every played run takes.
//!
//! **The fight has to be over first**, which is what makes these runs longer than any other scenario's:
//! a fight underway outranks the table — the enemy timeline runs on through a midboss, so a slow fight
//! reaches the boundaries of the waves that come after it and a chapter beginning inside a half-fought
//! fight is a retry point for neither. So the boss goes down and the stage plays on to the frame the
//! table names.
//!
//! Neither frame is written here: both come out of the table, so a stage tuned again is a scenario that
//! follows it rather than one that has to be edited with it.

use crate::fake::th06::{ATTACK_CHANGES, Fake, STAGES};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::RunStart;
use orb_core::game::th06::chapters::MIDSTAGE;
use orb_core::game::th06::image::{Screen, item};
use orb_sim::keys;

/// A full run of this game: Normal, Reimu A, from stage one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// And the Extra, which is a run of its own: the difficulty the game's own screens count fifth, and the
/// stage after the [`STAGES`] a full run goes through.
///
/// The stage is what picks the row of the table the run's chapters come out of — see [`MIDSTAGE`], whose
/// last row is the Extra's — and the difficulty is what names the file it is kept in.
fn the_extra() -> RunStart {
    RunStart {
        difficulty: 4,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: STAGES,
    }
}

/// The frame the table's first boundary for that stage falls on, out of the table itself.
///
/// # Panics
/// On a row with nothing in it: a stage the table has no boundary for is one this scenario has nothing
/// to wait for.
fn first_boundary(stage: i32) -> i32 {
    MIDSTAGE[stage as usize]
        .first()
        .unwrap_or_else(|| panic!("a boundary in stage {}'s row of the table", stage + 1))
        .frame
}

/// How long the fight's last attack runs before the boss is beaten, counted from [`ATTACK_CHANGES`].
///
/// Past the floor on how short a chapter may be — `chapter.rs`'s `MIN_CHAPTER_FRAMES`, a second — so that
/// the waves handed back when the fight is won are a chapter of their own. Beaten inside the floor they
/// are not, and then the chapter the table's boundary begins is a different number.
const LAST_ATTACK_LASTS: u32 = 120;

/// A launch of `run`, started from the title menu's `from` and played into its stage with the mode
/// chosen and the fight behind it.
///
/// The fight is this game's own script — a boss at `BOSS_ARRIVES` with a card, and its next attack at
/// [`ATTACK_CHANGES`] — and it is beaten [`LAST_ATTACK_LASTS`] frames after the last of that.
fn played_past_its_fight(name: &str, run: RunStart, from: i32) -> Box<Fake> {
    let game = Fake::attach(name, run, |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    // The item the run is started from, which orb asks the same question over whichever of the three it
    // is: that is what makes the Extra a run with chapters like any other.
    for _ in 0..from {
        game.press(keys::DOWN);
    }
    assert_eq!(
        game.image().front_end_now().cursor,
        from,
        "the cursor is not on the item this run is started from",
    );
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode chosen", || {
        game.log().said("mode: answered on the keyboard")
    });
    game.frames_until("the shot type select", 90, || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
    // The stage *built*, which is one frame past the scene being the one a stage runs in: the front end
    // writes the scene it wants and the supervisor builds the manager on the frame after, so the number
    // the stage's build raises is not there yet on the frame `playing` first answers true.
    game.frames_until("the stage built", 8, || {
        let state = game.state();
        state.playing && state.stage_frames >= 1
    });
    assert_eq!(
        game.state().stage,
        run.stage,
        "the stage the run was started for is not the one being played",
    );

    let fight_over = ATTACK_CHANGES + LAST_ATTACK_LASTS;
    game.frames_until("the fight's last attack", fight_over + 60, || {
        game.state().stage_frames >= fight_over
    });
    game.beats_its_boss();
    game.frame();
    // The waves handed back are a chapter of their own, which is what says the fight really is over —
    // and it is the chapter the table's boundary is counted from below.
    assert!(
        game.log().said("): the midboss was beaten"),
        "the fight being won was not a chapter of the waves it handed back:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// How many chapters the run has had by the time the table's boundary is reached: the stage's own start,
/// the boss arriving, the card it puts up, the attack after that card, and the waves handed back when the
/// fight was won.
///
/// Counted, because what a fixed number says is that nothing else began a chapter in between — the frames
/// from the fight to the boundary are the table's alone.
const CHAPTERS_BEFORE_IT: u32 = 5;

/// Runs `game` to the frame the table names for its stage, and asserts the chapter that began there is
/// the table's — by the frame, by the clock the table is written in, and by the cause the log names.
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
}

/// The boundary stage one's row holds begins a chapter in a run of the game.
#[test]
fn the_boundary_the_table_holds_for_a_stage_begins_a_chapter() {
    in_its_own_process(|| {
        let game = played_past_its_fight("table-boundary", the_run(), item::GAME_START);
        plays_to_the_boundary(&game, the_run().stage);
    });
}

/// And the Extra reads the row of its own, which is the last of the table's seven: it is a stage like
/// any other to everything above `Game`, and the stage number is the whole of what picks its row.
///
/// The run is kept under a name of its own as well, which is what a chapter written down for it says:
/// the Extra is `extra-reimu-a`, so a session left in it is offered back in the Extra and not in the
/// Normal run of the same shot.
#[test]
fn the_extra_reads_the_row_the_table_has_for_it() {
    in_its_own_process(|| {
        let game = played_past_its_fight("table-boundary-extra", the_extra(), item::EXTRA);
        plays_to_the_boundary(&game, the_extra().stage);
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec!["extra-reimu-a".to_owned()],
            "the Extra's chapters were not written down under a name of the Extra's",
        );
    });
}
