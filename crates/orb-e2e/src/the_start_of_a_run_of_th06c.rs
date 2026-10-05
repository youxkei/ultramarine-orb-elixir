//! **What a run of th06c starts with**, as 0x14002cc00 and the stage callback's run-start branch write it: the
//! lives and the bombs out of the configuration — two and three for the Extra and for practice alike
//! (0x14002cc1b) — the track stopped only where the configuration plays the wav tracks (0x14002cc70), and the
//! count of extends paid for at four for the Extra and nought for the rest (0x1400242d0 under `[+0x20] > 5`).
//!
//! Without chapters throughout, so that the numbers are the game's and none of orb's.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06c::image::MUSIC_AS_WAV;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A configuration other than the run's two and three, so that which of the two a run took is plain.
const CONFIGURED: (u8, u8) = (5, 1);
const TWO_AND_THREE: (i8, i8) = (2, 3);

const NORMAL: i32 = 1;
const EXTRA: i32 = 4;
const THE_EXTRAS: i32 = 6;

/// The configuration's music played as MIDI, which a run's start leaves to play on.
const MUSIC_AS_MIDI: u8 = 2;
const A_TRACK: i32 = 3;

fn a_launch(name: &str) -> Box<Fake> {
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
    game.image()
        .the_configuration_gives(CONFIGURED.0, CONFIGURED.1);
    game
}

#[test]
fn a_run_of_th06c_starts_with_the_lives_and_the_bombs_the_configuration_gives() {
    in_its_own_process(|| {
        let game = a_launch("th06c-a-run-starts-configured");
        game.image().the_difficulty_is(NORMAL);
        game.image().is_not_practising();
        game.in_a_run_in_the_stage(0);
        assert_eq!(
            game.image().lives_and_bombs(),
            (CONFIGURED.0 as i8, CONFIGURED.1 as i8)
        );
    });
}

#[test]
fn a_practice_run_of_th06c_starts_with_two_lives_and_three_bombs_whatever_the_configuration() {
    in_its_own_process(|| {
        let game = a_launch("th06c-a-practice-run-starts");
        game.image().the_difficulty_is(NORMAL);
        game.image().is_practising();
        game.in_a_run_in_the_stage(1);
        assert_eq!(game.image().lives_and_bombs(), TWO_AND_THREE);
    });
}

#[test]
fn the_extra_of_th06c_starts_with_its_count_of_extends_at_four() {
    in_its_own_process(|| {
        let game = a_launch("th06c-the-extra-starts");
        game.image().the_difficulty_is(EXTRA);
        game.image().is_not_practising();
        game.in_a_run_in_the_stage(THE_EXTRAS);
        assert_eq!(game.image().lives_and_bombs(), TWO_AND_THREE);
        assert_eq!(game.image().the_extra_lives_the_score_paid_for(), 4);
    });
}

#[test]
fn a_normal_run_of_th06c_starts_with_its_count_of_extends_at_nought() {
    in_its_own_process(|| {
        let game = a_launch("th06c-a-normal-run-starts");
        game.image().the_difficulty_is(NORMAL);
        game.image().is_not_practising();
        game.in_a_run_in_the_stage(0);
        assert_eq!(game.image().the_extra_lives_the_score_paid_for(), 0);
    });
}

#[test]
fn a_run_of_th06c_starts_with_the_track_stopped_where_the_music_is_the_wav_tracks() {
    in_its_own_process(|| {
        let game = a_launch("th06c-a-run-stops-the-wav");
        game.image().the_music_is_played_as(MUSIC_AS_WAV);
        game.image().plays_the_track(A_TRACK);
        game.in_a_run_in_the_stage(0);
        assert_eq!(game.image().the_track_playing(), -1);
    });
}

#[test]
fn a_run_of_th06c_leaves_the_track_alone_where_the_music_is_midi() {
    in_its_own_process(|| {
        let game = a_launch("th06c-a-run-leaves-the-midi");
        game.image().the_music_is_played_as(MUSIC_AS_MIDI);
        game.image().plays_the_track(A_TRACK);
        game.in_a_run_in_the_stage(0);
        assert_eq!(game.image().the_track_playing(), A_TRACK);
    });
}
