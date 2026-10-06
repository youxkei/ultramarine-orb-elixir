//! **The clock of th11's stage script stands still under its boss.**
//!
//! What orb reads as a stage's script frames is the count of the stage's frames on which none of its own scripts —
//! the enemies with no body, `main` and what it spawns to run the stage — stood on an instruction that waits (see
//! `Th11::script_frames`). That count is the script's own and not the stage's: the instruction a script waits for
//! its boss on, `0x154`, holds it where it is for the whole of the fight (0x415b1e through 0x415b30) — measured on
//! stage 3 practised alone, whose `main` stood on it from the midboss's arrival to its end and through the boss —
//! while the stage's frames at `0x4a5734` go on.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the stage lasts, and the frame of it its boss is up by: the boss fights for the stage's last 900.
const STAGE_FRAMES: u32 = 3_600;
const THE_BOSS_IS_UP_BY: u32 = STAGE_FRAMES - 900 + 10;

/// The frames the clock is watched for under the boss.
const WATCHED: u32 = 60;

#[test]
fn th11s_script_clock_stands_still_under_its_boss() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-script-clock",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                config.chapters = false;
            },
        );
        game.fires_at_the_player(false);
        game.stages_last(STAGE_FRAMES);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        // The clock going on before the boss, which is what makes it standing still under her say anything.
        game.frames_until("the stage's first frames", 240, || {
            game.state().in_run && game.state().stage_frames > 0
        });
        let early = game.state();
        game.frames(WATCHED);
        assert!(
            game.state().script_frames > early.script_frames,
            "the stage script's clock stood still before the boss, so this test is about nothing",
        );
        game.frames_until("the boss up", STAGE_FRAMES * 2, || {
            game.state().stage_frames >= THE_BOSS_IS_UP_BY
        });
        assert!(
            game.state().boss_present,
            "no boss is up, so this test is about nothing"
        );
        let before = game.state();
        game.frames(WATCHED);
        let after = game.state();
        assert!(
            after.stage_frames > before.stage_frames,
            "the stage stood still, so this test is about nothing",
        );
        assert_eq!(
            after.script_frames, before.script_frames,
            "the stage script's clock went on under its boss",
        );
    });
}
