//! **What a stage transition of th06nc carries, and what only the start of a run puts in place.**
//!
//! The game manager's own build is `0x14003a9c0`, one function reached two ways, and the whole of the
//! difference is its first condition: `0x14003a9ec` reads the supervisor's state wanted against
//! `STATE_NEXT_STAGE` and takes the stage-following path at `0x14003af12`, where a run's first stage
//! falls through. That path writes none of the run's numbers — see `Th06nc::numbers`, whose seven fields
//! are the seven it leaves alone — and the tail **both** paths reach raises the stage word by one at
//! `0x14003af45` and copies the generator into the stage's own seed at `0x14003b171`.
//!
//! **And the score is one field here and not two.** `0x14003af19` copies the manager's `+0x938` into the
//! score a chapter records, which reads as a panel being caught up to a run — but two launches of
//! `orb_real`'s `a_chapter_of_the_stage_after_the_first` read `+0x938` as nought across a boundary while
//! the run's score stood at 2,555,010 and 3,188,720, so there is no second field for a transition to
//! carry and none laid out. See `game::th06nc::game_manager::SCORE`.
//!
//! **Read out of the game's own code rather than off this tree**, because the laid-out game is both the
//! writer and the reader of the memory orb is driven through: a transition that put a stage's numbers
//! back would be a transition every e2e test here agreed with.
//!
//! **And the rank is not among what this asks about**, where th06c's module asks about it and the step
//! towards it: the byte this game's manager keeps a rank in is unread — see `th06nc/asm.md`'s *What is
//! not established*.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below starts in and the one it walks into, **as this game counts its own**, which
/// is from one: what the log says and what `game_manager::STAGE` holds while that stage runs.
const THE_FIRST_STAGE: i32 = 1;
const THE_NEXT_STAGE: i32 = 2;

/// And the same two as `State::stage` counts them, which is from nought — **and which is also what
/// `Fake::in_a_run_in_the_stage` is handed**: the stage's own build counts the number in that field on,
/// `0x14003af45` doing it on both of its paths, so the number written before it is the nought-counted
/// one. See `fake::th06nc::stage_begun`.
const THE_FIRST_STAGE_READ: i32 = THE_FIRST_STAGE - 1;
const THE_NEXT_STAGE_READ: i32 = THE_NEXT_STAGE - 1;

/// How long each stage lasts here, in the game's own frames: long enough for a run's numbers to be spent
/// inside the first one, and no further than it has to be.
const STAGE_FRAMES: u32 = 330;

/// The frames the walk into the next stage is given past the first stage's last.
const A_TRANSITION: u32 = 60;

/// And the frames a stage's own start is given to settle into a chapter, which is a second on top of the
/// wait itself as this test's wall.
const A_SETTLE: u32 = 120;

/// **The numbers this run is carrying when it crosses the boundary**, none of them a stage's own: lives
/// and bombs spent down from what the configuration gives, a shot part-way up its power, and a score
/// nothing of the boundary writes.
const LIVES: i8 = 1;
const BOMBS: i8 = 1;
const POWER: u16 = 96;
const SCORE: u64 = 1_234_560;
const POINT_ITEMS: u16 = 12;
/// **What the next point item pays, which is a field of its own beside the count above** — the item
/// walk steps it while the power is full and a run's own build writes it nowhere, so it crosses a
/// boundary the way the rest of these do. See `game::th06nc::game_manager::POINT_ITEM_VALUE`.
const POINT_ITEM_VALUE: u8 = 23;
const EXTRA_LIVES: i8 = 2;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// A run in its first stage, with nothing fired at the player: a death in this mode is a retry offered,
/// and a question nobody answers is a game standing still.
fn in_its_first_stage(name: &str) -> Box<Fake> {
    let game = launched(name);
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(THE_FIRST_STAGE_READ);
    game
}

/// The run walked into the stage after the one it was in, stopping on the frame that stage was built on.
fn walks_into_the_next_stage(game: &Fake) {
    game.frames_until(
        "the stage after the first",
        STAGE_FRAMES + A_TRANSITION,
        || game.state().stage == THE_NEXT_STAGE_READ,
    );
}

/// A run carries its lives, its bombs and its power across a stage boundary.
#[test]
fn a_run_of_th06nc_carries_its_lives_bombs_and_power_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06nc-a-transition-the-numbers");
        game.image().has_lives_and_bombs(LIVES, BOMBS);
        game.image().shot_is_at_power(POWER);
        walks_into_the_next_stage(&game);

        assert_eq!(
            (
                game.image().lives_and_bombs(),
                game.image().the_shots_power()
            ),
            ((LIVES, BOMBS), POWER),
            "the stage the run walked into was handed numbers of its own",
        );
    });
}

/// And its score, its point items, what the next point item pays and the lives the score has paid for,
/// which are the rest of what a run carries between its stages.
#[test]
fn a_run_of_th06nc_carries_its_score_items_and_extra_lives_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06nc-a-transition-the-score");
        game.image().the_run_has_scored(SCORE, POINT_ITEMS);
        game.image().a_point_item_is_worth(POINT_ITEM_VALUE);
        game.image().the_score_has_paid_for(EXTRA_LIVES);
        walks_into_the_next_stage(&game);

        assert_eq!(
            (
                game.image().the_score(),
                game.image().the_point_items(),
                game.image().what_a_point_item_is_worth(),
                game.image().the_extra_lives_the_score_paid_for()
            ),
            (SCORE, POINT_ITEMS, POINT_ITEM_VALUE, EXTRA_LIVES),
            "the stage the run walked into was handed a score of its own",
        );
    });
}

/// A transition is not the run ending: every frame of it reads as a run in progress.
///
/// **Which is what this game's reading of a run has to survive**: a stage of th06nc is the front end's
/// own absence and the player's job being in the chain, so a frame between two stages — the manager
/// rebuilding and the player's job gone — is exactly the frame a narrower reading would call the run
/// over.
#[test]
fn a_stage_transition_of_th06nc_is_not_the_run_ending() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06nc-a-transition-not-an-ending");
        assert_eq!(
            game.state().stage,
            THE_FIRST_STAGE_READ,
            "the run did not begin in the stage it was put in",
        );
        // Every frame from the last of the first stage to the first of the next, which is where a reading
        // that asked only for the stage's own jobs would answer that the run had gone.
        game.frames_until(
            "the first stage's last frames",
            STAGE_FRAMES + A_TRANSITION,
            || game.state().stage_frames + 2 >= STAGE_FRAMES,
        );
        for _ in 0..A_TRANSITION {
            game.frame();
            let state = game.state();
            assert!(
                state.in_run,
                "a frame of the transition reads as no run at all: {state}",
            );
        }
        assert_eq!(
            game.state().stage,
            THE_NEXT_STAGE_READ,
            "the run never walked into the stage after the first",
        );
    });
}

/// And the stage the run walked into gets a chapter of its own, which is what makes a transition a place
/// the mode can go back to.
#[test]
fn the_stage_of_th06nc_a_run_walks_into_is_given_a_chapter_of_its_own() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06nc-a-transition-the-chapter");
        let log = game.log();
        walks_into_the_next_stage(&game);
        game.frames_until("the stage's own first chapter", A_SETTLE, || {
            log.said(&format!("stage {THE_NEXT_STAGE} chapter 1 "))
        });
        assert_eq!(
            game.state().stage,
            THE_NEXT_STAGE_READ,
            "the chapter was taken somewhere other than the stage the run walked into",
        );
    });
}

/// And the score file is not read again for it: this game reads its own once, at the startup orb
/// brackets.
#[test]
fn a_stage_transition_of_th06nc_makes_no_read_of_the_score_file() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06nc-a-transition-the-score-file");
        let reads = || -> usize {
            game.score_file_opens()
                .iter()
                .filter(|(_, write)| !write)
                .count()
        };
        let before = reads();
        walks_into_the_next_stage(&game);
        assert_eq!(
            reads(),
            before,
            "the stage the run walked into read the score file: {:?}",
            game.score_file_opens()
        );
    });
}
