//! **What a stage transition of th08 carries, and what only the start of a run puts in place.**
//!
//! The run manager's added callback is `0x43aaf4`, which starts one build reached two ways: 0x43aafd reads
//! the scene wanted, and **3, 0xb and 0xc are a stage carrying a run on** (0x43ab24 leaves `0x17ce8bc` at
//! nought) where anything else is a run starting (0x43ab18 writes 1). The build on the thread branches on
//! that flag at 0x43ac89 through 0x43acb8, together with bits 14 and 0 of the run kind and the difficulty.
//!
//! **What only the start takes is the run's numbers put in place** — the run's clock back to nothing
//! (0x43ae88) and the lives, the bombs, the power and the score the front end chose. **What the transition
//! takes is the score handed to the panel**: 0x43ac89 through 0x43acb8 send a stage carried on — no
//! practice, no Extra — to 0x43b13b, which copies the score into the copy the panel draws (0x43b14a); and
//! every build then zeroes the score (0x43b808), which the run manager's next update puts back up to that
//! copy (0x43a6e9 through 0x43a713). The lives, the bombs and the power are written nowhere on that path —
//! which is what a run *is*: six stages with one set of numbers walked through them.
//!
//! **Read out of the game's own code rather than off this tree**, because the laid-out game is both the
//! writer and the reader of the memory orb is driven through: a transition that put a stage's numbers back
//! would be a transition every e2e test here agreed with. And **the score is why this is worth a test**:
//! read off the word the build zeroes, every stage after a run's first was written down as `score: 0` and
//! session 11's pick-up of stage 2 landed 46,534,890 short — see `Th08::numbers`.

use crate::fake::th08::{FIRST_STAGE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run walks into, orb's own 0-based number.
const THE_NEXT_STAGE: i32 = FIRST_STAGE + 1;

/// The frames a stage's build and the walk into it are given: the build waits out thirty frames of the
/// run manager's count on the supervisor's thread — see `Th08`'s `A_BUILD_STANDS_FOR` — and a chapter's
/// settle more.
const A_TRANSITION: u32 = 600;

/// **The numbers this run is carrying when it crosses the boundary**, none of them a stage's own: lives and
/// bombs spent down from what the front end gives, a shot part way up its power, and a score the panel's
/// copy has not caught up with.
const LIVES: f32 = 1.0;
const BOMBS: f32 = 1.0;
const POWER: f32 = 72.0;
const SCORE: u32 = 46_534_890;
const THE_PANEL_WAS_SHOWING: u32 = 46_534_000;

/// **A run in its first stage, placed rather than walked to**, its stage's clocks the stage's own and nothing
/// fired at the player: what this is about is the build between two stages, and a stage firing at a player
/// nobody moves is a death and a retry offered.
fn in_its_first_stage(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames_until("the first stage", 60, || game.state().in_run);
    game
}

/// The run walked into the stage after the one it was in — the run built again, as the run manager asks
/// for it at 0x439ddc — and stopping once that stage's build is over.
///
/// **The build waited for as it starts and then as it ends**, and not off the stage's number: the run
/// manager writes the next stage's number before it asks for the build, so a wait on the number alone ends
/// on the frame the build was asked for and nothing of it had run.
fn walks_into_the_next_stage(game: &Fake) {
    game.moves_on_to_the_stage_pressing_past_its_end(THE_NEXT_STAGE);
    game.frames_until("the stage after the first, built", A_TRANSITION, || {
        !game.image().the_loader_is_running()
    });
    assert_eq!(
        game.state().stage,
        THE_NEXT_STAGE,
        "the build was of a stage other than the one after the first",
    );
}

/// A run carries its lives, its bombs and its power across a stage boundary.
#[test]
fn a_run_of_th08_carries_its_lives_bombs_and_power_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th08-a-transition-the-numbers");
        game.image().has_lives_and_bombs(LIVES, BOMBS);
        game.image().shot_is_at_power(POWER);
        walks_into_the_next_stage(&game);

        let state = game.state();
        assert_eq!(
            (state.lives, state.bombs, state.power),
            (LIVES as i8, BOMBS as i8, POWER as u16),
            "the stage the run walked into was handed numbers of its own",
        );
    });
}

/// **And the score handed to the panel**: the copy it draws takes the run's score, and the score itself goes
/// to nought — which is the one thing the transition does to the numbers — until the run manager's first
/// update of the stage puts it back up to that copy.
#[test]
fn a_stage_transition_of_th08_hands_the_runs_score_to_the_panel() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th08-a-transition-the-score");
        game.image()
            .the_score_and_the_panels_copy_are(SCORE, THE_PANEL_WAS_SHOWING);
        walks_into_the_next_stage(&game);

        assert_eq!(
            game.image().the_score_and_the_panels_copy(),
            (0, SCORE),
            "the build did not hand the run's score to the panel's copy and put the score to nought",
        );
        game.frame();
        assert_eq!(
            game.image().the_score_and_the_panels_copy(),
            (SCORE, SCORE),
            "the stage's first update did not put the score back up to the panel's copy",
        );
    });
}

/// **And a stage carried on to waits for a press before it is built** — the tally the stage's end shows: the
/// stage's end puts bits 5 and 6 of the run's kind up (0x439bc7), the build's thread waits past its own frames
/// for them to read nothing (0x43b88b), and only the run manager's tail takes them down, on a press of the
/// shot or the enter (`0x1001`, 0x439f3c) — which the supervisor reads on every frame of the build. Measured
/// on the real game: a clear's move from stage 1 to stage 2 came on the bot's own shot pressed again 34
/// frames before the stage word moved.
#[test]
fn a_stage_of_th08_carried_on_to_waits_for_a_press_before_it_is_built() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th08-a-transition-waits-for-a-press");
        game.moves_on_to_the_stage(THE_NEXT_STAGE);
        game.frames_until(
            "the build of the stage after the first",
            A_TRANSITION,
            || game.image().the_loader_is_running(),
        );
        game.frames(A_TRANSITION);
        assert!(
            game.image().the_loader_is_running(),
            "the stage after the first was built with nothing pressed",
        );
        game.press(orb_sim::keys::Z);
        game.frames_until("the stage after the first, built", A_TRANSITION, || {
            !game.image().the_loader_is_running()
        });
    });
}

/// **And a run orb plays itself is pressed past that tally**: the rules press what a stage wants and hold the
/// shot, which is no press, so orb taps the decide while a stage's end stands — see
/// `Game::a_stages_end_waits_for_a_press`.
///
/// **This does not fail without the tap**: the laid-out stage leaves the rules cycling the shot through the
/// build, where the real game's clear held it from the frame after its first stage's clock stopped — the tap
/// is for a stage's end landing mid-hold, which the laid-out game does not produce.
#[test]
fn a_run_of_th08_orb_plays_is_pressed_past_its_stages_end() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-transition-pressed-past-by-orb",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { stage: 2, chapter: 1 }\n  give_up_after: 6000\n".to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.fires_at_the_player(false);
        game.stages_last(A_SHORT_STAGE);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.plays_its_own_stage();
        game.frames_until(
            "the stage after the first, built",
            A_SHORT_STAGE + A_TRANSITION,
            || game.state().stage == THE_NEXT_STAGE && !game.image().the_loader_is_running(),
        );
    });
}

/// A stage short enough to be over in a test.
const A_SHORT_STAGE: u32 = 300;

/// A transition is not the run ending: every frame of it reads as a run in progress.
#[test]
fn a_stage_transition_of_th08_is_not_the_run_ending() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th08-a-transition-not-an-ending");
        game.moves_on_to_the_stage(THE_NEXT_STAGE);
        let mut built = false;
        // The shot tapped through it, the tally waiting on a press.
        let mut down = false;
        for _ in 0..A_TRANSITION {
            down = !down;
            game.keyboard().set(orb_sim::keys::Z, down);
            game.frame();
            let state = game.state();
            assert!(
                state.in_run,
                "a frame of the transition reads as no run at all: {state}",
            );
            let building = game.image().the_loader_is_running();
            if built && !building {
                return;
            }
            built |= building;
        }
        panic!("the run never walked into the stage after the first");
    });
}

/// And the stage the run walked into gets a chapter of its own, which is what makes a transition a place the
/// mode can go back to.
#[test]
fn the_stage_of_th08_a_run_walks_into_is_given_a_chapter_of_its_own() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th08-a-transition-the-chapter");
        let log = game.log();
        walks_into_the_next_stage(&game);
        game.frames_until("the stage's own first chapter", A_TRANSITION, || {
            log.said(&format!(
                "stage {} chapter 1 (stage start)",
                THE_NEXT_STAGE + 1
            ))
        });
        assert_eq!(
            game.state().stage,
            THE_NEXT_STAGE,
            "the chapter was taken somewhere other than the stage the run walked into",
        );
    });
}
