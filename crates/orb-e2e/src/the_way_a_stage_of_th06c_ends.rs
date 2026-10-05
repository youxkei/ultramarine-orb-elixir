//! **Where the end of a stage of th06c takes the run**, as the message script's end-of-stage instruction decides
//! it (0x140028410's case 0xb): the next stage for a run not in practice whose stage word is below 5, or 5 on any
//! difficulty but Easy; the title for a replay; the ending for every difficulty but the Extra's; and the screen
//! a run ends at for the Extra and for practice, with no ending before it.
//!
//! **And the stage word kept between the stages**: nothing in the exe writes it nought, and the stage callback
//! raises it by one (0x1400242d0's tail).
//!
//! Without chapters throughout, so that nothing of orb's holds the game at a stage's end.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, and the frames past it a stage's end is given to land.
const STAGE_FRAMES: u32 = 200;
const AFTER_IT: u32 = 10;

/// The game's own difficulties, `[0x1403ed7d8]`.
const EASY: i32 = 0;
const NORMAL: i32 = 1;
const EXTRA: i32 = 4;

/// The stages, counted from nought as `Fake::in_a_run_in_the_stage` counts them: the fifth, the sixth, and the
/// Extra's.
const THE_FIFTH: i32 = 4;
const THE_SIXTH: i32 = 5;
const THE_EXTRAS: i32 = 6;

fn a_run_on(name: &str, difficulty: i32, practice: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.chapters = false;
        },
    );
    game.fires_at_the_player(false);
    game.stages_last(STAGE_FRAMES);
    game.image().the_difficulty_is(difficulty);
    if practice {
        game.image().is_practising();
    } else {
        game.image().is_not_practising();
    }
    game
}

/// **What the stage's end came to**, read every frame until one of the three ways out has been taken: another
/// stage built, the ending up, or the screen a run ends at up — or the title back.
#[derive(Debug, PartialEq)]
enum Out {
    NextStage,
    Ending,
    ScreenARunEndsAt,
    Title,
}

/// **An ending seen either way it can be**: up across a frame, or run out inside one by orb's own skip, which
/// leaves nothing of it standing between two frames but the line it writes.
const AN_ENDING_SKIPPED: &str = "ending skipped";

fn where_it_went(game: &Fake) -> Out {
    let word = game.image().the_games_stage_word();
    for _ in 0..STAGE_FRAMES + AFTER_IT {
        game.frame();
        let image = game.image();
        if game.state().in_ending || game.log().said(AN_ENDING_SKIPPED) {
            return Out::Ending;
        }
        if image.the_state_the_screen_a_run_ends_at_is_in() != 0 {
            return Out::ScreenARunEndsAt;
        }
        if image.the_scene_now() == image.scene_of_the_front_end() {
            return Out::Title;
        }
        if image.the_games_stage_word() == word + 1 {
            return Out::NextStage;
        }
    }
    panic!(
        "the stage's end went nowhere in {} frame(s)",
        STAGE_FRAMES + AFTER_IT
    );
}

#[test]
fn an_easy_run_of_th06c_ends_after_its_fifth_stage() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-easy-ends", EASY, false);
        game.in_a_run_in_the_stage(THE_FIFTH);
        assert_eq!(where_it_went(&game), Out::Ending);
    });
}

#[test]
fn a_normal_run_of_th06c_goes_on_from_its_fifth_stage_to_its_sixth() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-normal-goes-on", NORMAL, false);
        game.in_a_run_in_the_stage(THE_FIFTH);
        assert_eq!(where_it_went(&game), Out::NextStage);
    });
}

#[test]
fn the_extra_of_th06c_ends_at_the_screen_a_run_ends_at_with_no_ending() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-the-extra-ends", EXTRA, false);
        game.in_a_run_in_the_stage(THE_EXTRAS);
        assert_eq!(where_it_went(&game), Out::ScreenARunEndsAt);
    });
}

#[test]
fn a_practice_run_of_th06c_ends_with_its_stage_at_the_screen_a_run_ends_at() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-practice-ends", NORMAL, true);
        game.in_a_run_in_the_stage(1);
        assert_eq!(where_it_went(&game), Out::ScreenARunEndsAt);
    });
}

#[test]
fn a_replay_of_th06c_played_to_its_last_stages_end_goes_to_the_title_with_its_flag_down() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-a-replay-ends", NORMAL, false);
        game.watches_a_replay_of_the_stage(THE_SIXTH);
        assert_eq!(where_it_went(&game), Out::Title);
        assert!(
            !game.image().is_a_replay_playing(),
            "the title came back with the playback flag still up",
        );
    });
}

#[test]
fn a_replay_of_th06c_goes_on_from_a_stage_before_its_last() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-a-replay-goes-on", NORMAL, false);
        game.watches_a_replay_of_the_stage(1);
        assert_eq!(where_it_went(&game), Out::NextStage);
    });
}

#[test]
fn the_stage_word_of_th06c_holds_the_stage_ended_until_the_next_is_built() {
    in_its_own_process(|| {
        let game = a_run_on("th06c-the-stage-word-between", NORMAL, false);
        game.in_a_run_in_the_stage(1);
        let word = game.image().the_games_stage_word();
        for _ in 0..STAGE_FRAMES + AFTER_IT {
            game.frame();
            let now = game.image().the_games_stage_word();
            assert!(
                now == word || now == word + 1,
                "the stage word read {now} between the stages {word} and {}",
                word + 1,
            );
            if now == word + 1 {
                return;
            }
        }
        panic!("the next stage was never built");
    });
}
