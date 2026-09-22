//! **The stepping keys over th08: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage_of_th06c.rs` asks of that game, asked of this one. Both go through
//! `Chapters::previous_start` and `next_start`, whose whole job is that a step lands **strictly** either side
//! of where the game is — so a press moves rather than staying put, and a frame or two past a boundary steps
//! back onto it.
//!
//! **The boundaries are the stage's table and its fight**: the stage's own start, the midstage table's one
//! row of stage 1, the stage's own boss arriving, and the card she puts up and the nonspell after it — see
//! `fake::th08`'s `THE_BOSS_FIGHTS_FOR` and `HER_CARD`. **And the
//! wall at the far end is her going down**, which this game's stage does some frames before its next stage is
//! built — the scene staying the run's across the build, the way th07's does — so what a step forward comes
//! to rest on is the stage won.

use orb_config::LogLevel;
use orb_core::game::th08::chapters::MIDSTAGE;

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;
/// And the one held with a step to leave the stage altogether.
const ACROSS: u8 = orb_config::keys::SHIFT.0;

/// **How long a stage lasts here**: past the table's row of stage 1 and a fight after it — the laid-out
/// stage's boss arrives for its last 900 frames and goes 120 before its end.
const STAGE_FRAMES: u32 = 2_600;
/// And where in it she arrives, which the row has to come before for the row to be a chapter.
const THE_BOSS_ARRIVES_AT: i32 = STAGE_FRAMES as i32 - 900;

/// How many frames a wait here is given.
const FRAMES: u32 = STAGE_FRAMES * 3;

/// The script frame the table's one row of stage 1 is at.
fn the_tables_row() -> i32 {
    // What makes the stage's boundaries three before her fight: a second row before she arrives would be a
    // chapter between the row and her, and every count of presses below would be one out.
    let rows: Vec<i32> = MIDSTAGE[0]
        .iter()
        .map(|boundary| boundary.frame)
        .filter(|&frame| frame < THE_BOSS_ARRIVES_AT)
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "stage one's row of the table has {rows:?} before a boss arriving at {THE_BOSS_ARRIVES_AT}",
    );
    rows[0]
}

/// One press of a stepping key with nothing else held, and the frame it left the game on.
///
/// # Panics
/// Where the step wrote no line saying where it held the game, which is a press that did nothing.
fn steps(game: &Fake, key: u8) -> u32 {
    let held = game.log().written();
    game.press(key);
    let said = game.log().lines().split_off(held).join("\n  ");
    assert!(
        said.contains("step: held"),
        "the press of {key:#04x} held the game nowhere:\n  {said}",
    );
    game.state().stage_frames
}

/// **The launch a judging pass is**: the stepping keys on over a replay, placed rather than chosen off the
/// list — the list is `a_pointdevice_run_of_th08`'s — with nothing fired at the player, a replay being no
/// run to be hit in.
fn judging(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_stepping = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.image().the_run_reads_its_own_input();
    game.frames_until("the stage's own first chapter", FRAMES, || {
        game.log().said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The three boundaries of this stage, each waited for and the frame it began on kept.
fn the_boundaries(game: &Fake) -> (u32, u32, u32) {
    let log = game.log();
    let the_stages_own_start = game.state().stage_frames;
    game.frames_until("the table's first row", FRAMES, || log.said("chapter 2"));
    let the_row = game.state().stage_frames;
    game.frames_until("the boss's chapter", FRAMES, || log.said("chapter 3"));
    let the_boss = game.state().stage_frames;
    assert!(
        game.state().script_frames >= the_tables_row() && game.state().boss_present,
        "the chapters did not fall where the table and the fight put them: {the_row}, {the_boss}",
    );
    (the_stages_own_start, the_row, the_boss)
}

#[test]
fn a_step_inside_a_stage_of_th08_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = judging("th08-stepping-inside");
        let log = game.log();
        let (start, row, boss) = the_boundaries(&game);

        // Held a little way past the last of them: a step back from inside a chapter goes to that chapter's
        // own start, which is what makes a press the way onto the boundary just passed.
        game.frames(60);
        let past = game.state().stage_frames;
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the second press does not stay on the
        // frame the first landed on.
        assert_eq!(steps(&game, BACK), boss, "one press back");
        assert_eq!(steps(&game, BACK), row, "two presses back");
        assert_eq!(steps(&game, BACK), start, "three presses back");

        // And the press after that, which has nowhere left to go inside the stage.
        game.press(BACK);
        assert!(
            log.said("step: held at the stage's start"),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            start,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same two: what a step forward runs to is the next start where the
        // stage has one.
        assert_eq!(steps(&game, NEXT), row, "one press forward");
        assert_eq!(steps(&game, NEXT), boss, "two presses forward");
    });
}

/// **And a step forward does not leave the stage**: the stage won is a wall, and a step back from it still
/// goes where it always goes.
#[test]
fn a_step_forward_inside_a_stage_of_th08_does_not_leave_it() {
    in_its_own_process(|| {
        let game = judging("th08-stepping-the-wall");
        let (_, row, boss) = the_boundaries(&game);
        game.press(HOLD);

        // Forward over the rest of her fight first, which has two more starts: the card she puts up and the
        // nonspell after it.
        let card = steps(&game, NEXT);
        let after = steps(&game, NEXT);
        assert!(
            boss < card && card < after,
            "the steps forward over her fight did not each move on: {boss}, {card}, {after}",
        );

        let held = game.log().written();
        game.press(NEXT);
        assert!(
            !game.log().said_since(held, "stage 2 chapter 1"),
            "the step forward ran on into the stage after the one it was made in:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
        assert!(
            game.log().said_since(
                held,
                "step: stage 1 has ended; the across key with next or back leaves it"
            ),
            "the wall did not say which stage had ended:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
        // And back from that wall, which is what makes it a place to be rather than one a pass loses the
        // stage from: every chapter of the stage is still a snapshot orb holds.
        assert_eq!(steps(&game, BACK), after, "a step back from the wall");
        assert_eq!(steps(&game, BACK), card, "two presses back from the wall");
        assert_eq!(steps(&game, BACK), boss, "three presses back from the wall");
        assert_eq!(steps(&game, BACK), row, "four presses back from the wall");
    });
}

/// **And the way out of a stage is one this game declines**, which is what the across key finds: it says so
/// and the pass stays where it was. `Th08::jump_to_stage` answers false, no reading of how this game's own
/// playback is asked for another stage having been made.
#[test]
fn the_across_key_over_th08_says_the_replay_has_no_other_stage() {
    in_its_own_process(|| {
        let game = judging("th08-stepping-across");
        let log = game.log();
        let (_, row, boss) = the_boundaries(&game);
        game.frames(60);
        game.press(HOLD);

        let from = log.written();
        game.keyboard().set(ACROSS, true);
        game.press(NEXT);
        game.keyboard().set(ACROSS, false);
        assert!(
            log.said_since(from, "step: the replay has no stage"),
            "the across key was taken as a move this game can make:\n  {}",
            log.lines().split_off(from).join("\n  ")
        );

        // And the stage is still the one the pass was in, with its own boundaries to step between.
        assert_eq!(steps(&game, BACK), boss, "a step back after the refusal");
        assert_eq!(steps(&game, BACK), row, "two presses back after it");
    });
}
