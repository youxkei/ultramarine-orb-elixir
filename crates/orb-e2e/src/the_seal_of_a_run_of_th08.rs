//! **The decoys a run of th08's seal holds**: runs of dwords and floats among the run's numbers that the run
//! manager's update holds inside 6543 and 106543, breaking the seal — the sum's float written -9999.0 — where
//! one is outside (0x439bc7). A write of the run's numbers from outside that leaves one there is a run the
//! enemy manager stops.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.plays_its_own_stage();
    game.frames_until("the run", 60, || game.state().in_run);
    game.frames(10);
    game
}

#[test]
fn a_run_of_th08_starts_with_its_decoys_holding_its_seal() {
    in_its_own_process(|| {
        let game = in_a_run("th08-the-decoys-hold");
        assert!(
            game.image().the_run_is_as_it_was_sealed(),
            "a run's own numbers broke its seal",
        );
    });
}

#[test]
fn a_decoy_of_a_run_of_th08_out_of_its_range_breaks_the_seal() {
    in_its_own_process(|| {
        let game = in_a_run("th08-a-decoy-out-of-range");
        game.image().the_runs_first_decoy_is(0);
        game.frame();
        assert!(
            !game.image().the_run_is_as_it_was_sealed(),
            "a decoy out of its range left the seal holding",
        );
    });
}
