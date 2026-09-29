//! **The numbers a run of th06 is made with, put back where a run left in a stage after its first is picked
//! up.**
//!
//! What `the_numbers_a_run_of_th07_was_made_with.rs` asks of that game, asked of this one. A run picked up is a
//! run started again from the front end, so the stage it lands in is built down the branch of
//! `GameManager::AddedCallback` a run's *first* stage takes — `curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT`
//! — which is the branch that puts a fresh run's lives, bombs, power, deaths and rank in place; see
//! `a_stage_transition.rs`. **Every one of those is then orb's to write back**, which is
//! `Th06::put_the_numbers_back`: the numbers `resume::Numbers` carries out of the file, written where the game
//! has the stage built and has not yet updated it.
//!
//! **A first stage cannot say whether it did.** What a first stage begins with is what a fresh run begins
//! with, so a pick-up that wrote nothing lands on the same numbers there. A stage the run walked into begins
//! with what the stage before it left, and those are the numbers this sets apart from a fresh run's: a life
//! and two bombs spent, power collected, a death counted, the rank climbed and an extra life paid for.
//!
//! **The score and the generator are the landing's own line**, `resume: the landing is the frame that was
//! written down, field for field`, which holds them with the rest of `Reproduction`; what this reads is the
//! numbers that line does not carry.

use crate::fake::th06::{FRESH, Fake, INVULNERABLE_AFTER_SPAWNING, RANK_AT_A_RUNS_START, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::Playing;
use orb_sim::keys;

/// How long each of this game's stages runs here: past the settle a stage's first chapter waits out and the
/// frames its own start cannot be killed in, with room for the death the run is given up by to land inside
/// the second.
const STAGE_FRAMES: u32 = 1_200;

/// The frame of the second stage the death lands on: past the frames its own start cannot be killed in.
const DIES_AT: u32 = INVULNERABLE_AFTER_SPAWNING as u32 + 20;

/// The stage the run is left in, counted from zero the way everything above `Game` counts.
const THE_SECOND: i32 = 1;

/// What the first stage leaves the run with, each of them apart from what a fresh run starts with — see
/// [`FRESH`] and [`RANK_AT_A_RUNS_START`].
const LIVES: i8 = FRESH.0 - 1;
const BOMBS: i8 = FRESH.1 - 2;
const POWER: u16 = 48;
const DEATHS: i32 = 1;
const RANK: i32 = RANK_AT_A_RUNS_START + 4;
const EXTRA_LIVES: i8 = 1;

/// The numbers a stage of a run is played with that a pick-up has to put back: the lives, bombs, power and
/// deaths out of the game manager, and the rank and extra lives beside them.
fn numbers(game: &Fake) -> (i8, i8, u16, i32, i32, i8) {
    let run = game.image().playing_now();
    (
        run.lives,
        run.bombs,
        run.power,
        run.deaths,
        unsafe { Th06.reproduced() }.rank,
        game.image().extra_lives(),
    )
}

/// The hit and then タイトルに戻る, which leaves the chapter behind for the next launch to offer.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.dies();
    game.frames_until("the death seen", READS_KEYS_AFTER, || {
        log.said_since(from, "died in chapter")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
}

#[test]
fn a_run_of_th06_picked_up_in_a_stage_after_its_first_lands_with_the_numbers_that_stage_began_with()
{
    in_its_own_process(|| {
        let game = Fake::attach(
            "th06-the-numbers-a-run-was-made-with",
            the_run(),
            |config| {
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.stages_last(STAGE_FRAMES);
        game.in_a_pointdevice_run();

        // The first stage spending and gaining what a run does, all of it apart from a fresh run's numbers.
        let run = game.image().playing_now();
        game.image().playing(Playing {
            lives: LIVES,
            bombs: BOMBS,
            power: POWER,
            deaths: DEATHS,
            ..run
        });
        game.image().set_rank(RANK);
        game.image().set_extra_lives(EXTRA_LIVES);

        // Into the second stage, which carries all of them — see `a_stage_transition.rs` — and its own start
        // written down, which is the chapter the run is then left in.
        let from = log.written();
        game.frames_until("the stage after the first", STAGE_FRAMES + 60, || {
            game.state().stage == THE_SECOND
        });
        let began_with = numbers(&game);
        assert_eq!(
            began_with,
            (LIVES, BOMBS, POWER, DEATHS, RANK, EXTRA_LIVES),
            "the stage after the first did not begin with the numbers the first left, so this is about \
             nothing",
        );
        game.frames_until_a_thread("the second stage written down", || {
            log.said_since(from, "stage 2 chapter 1 (MIDSTAGE 1) at frame")
        });
        game.frames_until("the second stage's own first chapter", STAGE_FRAMES, || {
            log.said_since(from, "stage 2 chapter 1 (stage start)")
        });
        // Past the frames the stage's own start cannot be killed in, so that the death below lands.
        game.frames_until("the second stage played into", STAGE_FRAMES, || {
            game.state().stage_frames >= DIES_AT
        });
        gives_the_run_up(&game);

        // And picked up, which builds the stage as a run's first — a fresh run's numbers — and then writes
        // the file's over them.
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where it was written:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage,
            THE_SECOND,
            "the run was not picked up in the stage it was left in",
        );
        assert_eq!(
            numbers(&game),
            began_with,
            "the run came back with numbers other than the ones the stage it was left in began with",
        );
    });
}
