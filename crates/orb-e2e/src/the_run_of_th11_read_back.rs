//! **What th11's memory says a run is before one is started, and where the log of a launch of it goes.**
//!
//! `th11.rs` reads a run back — the scene, the stage, the difficulty and the run's own numbers — and holds the
//! title to reading as no run at all. What this adds is every other chase `Th11::read_state` makes, over a title
//! with nothing of a run built: a field the game has not built yet is reached through a pointer that is
//! nothing, and each has to come back as "none" rather than as a fault or a number out of an unbuilt object —
//! the boss's slot off the enemy manager, the card up, the conversation off the panel, the laser manager and the
//! ending — the way `the_run_of_th10_read_back.rs` holds that game's. **Nothing, on the real game too**: every
//! one of those pointers is past the `0x6400` bytes of `.data` the file holds, from `0x4a2000` to `0x4a8400`
//! (`0x4a8d6c`, `0x4a8d7c`, `0x4a8d84`, `0x4a8e94`), which the loader zeroes, and nothing but a stage's build
//! writes them.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// The log goes beside the game, which is the one thing about it a game decides: `orb.yaml` and the launcher
/// are there, so that is where somebody looks for what a run did.
#[test]
fn the_log_of_a_launch_of_th11_is_written_beside_the_game() {
    in_its_own_process(|| {
        let game = launched("th11-read-back-log");
        assert_eq!(
            game.log().path(),
            Some(game.dir().join("orb.log")),
            "the log is not beside the exe orb was injected into",
        );
        // The line every run opens with. A log without it cannot say whether the run even happened.
        assert!(game.log().said("---- new run ----"));
        // Appended to rather than started over: a run worth looking at is often over by the time anybody
        // looks.
        assert!(!game.log().restarted());
    });
}

/// Before a run is started, nothing in the game reads as one — and the chases that have nothing to chase come
/// back as nothing rather than faulting.
#[test]
fn nothing_is_a_run_of_th11_before_one_is_started() {
    in_its_own_process(|| {
        let game = launched("th11-read-back-title");
        game.its_title_menu_is_up();
        game.one_frame();
        let state = game.state();
        assert!(!state.playing, "the title menu reads as a stage running");
        assert!(!state.in_run);
        assert!(!state.in_game);
        assert!(!state.replay);
        assert!(!state.demo);
        assert!(!state.practice);
        assert!(!state.paused);
        assert!(!state.unsettled);
        assert!(!state.bombing);
        assert!(!state.in_ending);
        assert_eq!(state.ending_script, None);
        // The boss's slot, the card up, the conversation and the laser array: chases with nothing at the end
        // of them.
        assert!(
            !state.in_dialogue,
            "the title menu reads as a line of dialogue up"
        );
        assert!(!state.boss_present);
        assert_eq!(state.boss_life, None);
        assert_eq!(state.boss_attack_frames, None);
        assert_eq!(state.spellcard, None);
        assert_eq!(state.laser_count, 0);
        assert_eq!(state.bullet_count, 0);
        assert_eq!(state.enemy_count, 0);
        assert_eq!(state.deaths, 0);
    });
}
