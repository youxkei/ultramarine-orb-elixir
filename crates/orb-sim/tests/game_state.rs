//! Reading a whole `State` out of a game laid out by hand.
//!
//! The real `Th06::read_state` — every offset, every pointer chase, every derived flag — over a
//! simulated Windows holding the game's own addresses, with no game running and no Windows needed
//! for the memory. What this settles is the parse: a `State` built by hand in a test says only what
//! the test put in it, where one read out of an image says what orb would have read off the running
//! game.
//!
//! What it cannot settle is an offset that is wrong, since the image is laid out from the same
//! constants the reads use. Those are settled against the real game — see `DONE.md`.

use orb_core::game::Game;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{Image, Playing};

#[test]
fn a_game_with_nothing_written_reads_as_no_run_at_all() {
    let image = Image::laid_out();
    let _installed = image.enter();

    // Every field a freshly committed page reads as, which is what the front end looks like before
    // anything has been played: scene zero is not the scene a stage runs in.
    let state = unsafe { Th06.read_state() };
    assert_eq!(state.scene, 0);
    assert!(!state.playing);
    assert!(!state.in_run);
    assert!(!state.in_game);
    assert!(!state.replay);
    assert!(!state.demo);
    assert!(!state.practice);
    assert!(!state.paused);
    assert!(!state.in_ending);
    assert!(!state.boss_present);
    assert_eq!(state.boss_life, None);
    assert_eq!(state.spellcard, None);
    // The dialogue index is chased through `GuiImpl`, which is a heap pointer the game has not
    // written: the chase has to come back as "no dialogue" rather than fault.
    assert!(!state.in_dialogue);
}

#[test]
fn the_numbers_a_stage_is_played_with_come_back_as_they_were_written() {
    let image = Image::laid_out();
    let _installed = image.enter();

    let run = Playing {
        stage: 3,
        difficulty: 2,
        frames: 1886,
        script_frames: 742,
        seed: 0x4d2,
        deaths: 1,
        lives: 1,
        bombs: 2,
        power: 128,
        enemies: 7,
    };
    image.playing(run);

    let state = unsafe { Th06.read_state() };
    assert_eq!(state.scene, 2, "the scene the game runs a stage in");
    assert_eq!(state.wanted, 2);
    assert!(state.playing);
    assert!(state.in_run);
    assert!(
        state.in_game,
        "somebody is playing it: neither a demo nor a replay"
    );

    // Counted from zero above `Game`, although the game keeps stage four as a four.
    assert_eq!(state.stage, 3);
    assert_eq!(state.difficulty, 2);
    assert_eq!(state.stage_frames, 1886);
    assert_eq!(state.script_frames, 742);
    assert_eq!(state.random_seed, 0x4d2);
    assert_eq!(state.deaths, 1);
    assert_eq!(state.lives, 1);
    assert_eq!(state.bombs, 2);
    assert_eq!(state.power, 128);
    assert_eq!(state.enemy_count, 7);

    // Nothing was laid out for these, and each is its own chase: the bosses pointer is null, the
    // spell card flag is clear, and no laser is in use.
    assert!(!state.boss_present);
    assert_eq!(state.boss_life, None);
    assert_eq!(state.boss_attack_frames, None);
    assert_eq!(state.spellcard, None);
    assert_eq!(state.laser_count, 0);
}

#[test]
fn a_replay_being_watched_is_a_run_but_not_a_game_somebody_is_playing() {
    let image = Image::laid_out();
    let _installed = image.enter();
    image.playing(Playing::default());
    image.watching_a_replay();

    let state = unsafe { Th06.read_state() };
    assert!(state.replay);
    assert!(
        state.in_run,
        "a replay is a run: it has stages and chapters"
    );
    assert!(
        !state.in_game,
        "and it is not one somebody is playing, which is what decides whether orb acts on it"
    );
}
