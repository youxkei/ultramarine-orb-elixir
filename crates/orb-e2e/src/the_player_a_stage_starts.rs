//! **The player a stage starts, and how long nothing can kill them.**
//!
//! `Player::AddedCallback` — called from inside `GameManager::AddedCallback` by `Player::RegisterChain` —
//! leaves `playerState = PLAYER_STATE_SPAWNING`, `invulnerabilityTimer.SetCurrent(120)` and
//! `respawnTimer = 8`. **None of those three is what the stage is played with**, and reading only that
//! callback is how the number 120 gets believed: `Player::OnUpdate`'s spawning branch tests
//! `30 <= invulnerabilityTimer.AsFrames()`, which 120 already satisfies, so the very first update of the
//! stage flips the state to `PLAYER_STATE_INVULNERABLE`, zeroes the respawn timer and calls
//! `SetCurrent(240)`. The countdown below it then runs in that same update, leaving **239**.
//!
//! So a stage begins with **240 updates nothing can kill the player in**, and the 121st is not special.
//! `src/Player.cpp`, and `src/ZunTimer.hpp` for what `SetCurrent` and `Decrement` do to the count.
//! `PLAYER_INVULNERABLE_FRAMES` in `orb_core::game::th06` is the same 240 read from the other end — what
//! orb writes under the state every update of a `--clear` run.
//!
//! **The spawning state is not asserted here, and that is the finding rather than an omission.** It lasts
//! one update and `Player::OnUpdate` is the job that ends it, so `Th06::read_state` — which orb calls after
//! the whole chain walk — can never see it at a stage's start. What can see it is `resume::stage_begun`,
//! which runs inside the callback, and it reads the run's numbers and not the player's. The state is
//! written where the game writes it all the same, because the update that flips it is the update that
//! turns 120 into 240.
//!
//! **Read out of the game's own code rather than off this tree**, because the fake game is both the writer
//! and the reader of the memory orb is driven through: a stage that started its player killable would be
//! one every scenario here agreed with. See
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
//!
//! レガシーモード throughout, so nothing of orb's is between the bullet and the player: `--clear` writes
//! the state and the frames under it before every update, which is
//! `a_clear_on_demand.rs`'s subject and would answer this one's question for it.

use crate::fake::th06::{FRESH, Fake, INVULNERABLE_AFTER_SPAWNING, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Player;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    })
}

/// A stage's own first update leaves the player invulnerable with one frame of the count already spent.
#[test]
fn a_stage_starts_its_player_invulnerable_with_the_first_frame_already_spent() {
    in_its_own_process(|| {
        let game = launched("a-stages-player-invulnerable");
        // `in_a_legacy_run` stops on the frame the stage was built on, which is a frame the stage has
        // already been updated on — see `the_frame_a_scene_is_built_on.rs`.
        game.in_a_legacy_run();
        assert_eq!(
            game.image().player_now(),
            Player::Invulnerable,
            "the stage started its player in a state the hit test can kill",
        );
        assert_eq!(
            game.image().invulnerable_frames(),
            INVULNERABLE_AFTER_SPAWNING - 1,
            "the count under that state is not the one the spawning branch's own update leaves",
        );
    });
}

/// And nothing kills the player until it runs out, on the update it runs out on.
///
/// A bullet sitting on the player from the stage's first frame, so that every update runs the hit test
/// against a live one: what says the invulnerability is doing the work is that the death arrives on the
/// **240th** update of the stage and not on the first.
#[test]
fn nothing_kills_the_player_until_a_stages_invulnerability_runs_out() {
    in_its_own_process(|| {
        let game = launched("a-stages-player-a-bullet");
        game.in_a_legacy_run();
        game.puts_a_bullet_on_the_player();

        // Every update up to the last one of the count, each of them a hit test against a live bullet.
        let last = INVULNERABLE_AFTER_SPAWNING as u32 - 1;
        game.frames_until("the last update of the invulnerability", last + 60, || {
            game.state().stage_frames == last
        });
        let before = game.state();
        assert_eq!(
            (before.deaths, before.lives),
            (0, FRESH.0),
            "a life went to the bullet inside the frames nothing can kill the player in",
        );

        // And the one after it, which is where the count reaches nothing and the state goes back to
        // normal — inside the same update, and before the bullets are checked at chain priority 11.
        game.frame();
        let after = game.state();
        assert_eq!(
            (after.stage_frames, after.deaths, after.lives),
            (last + 1, 1, FRESH.0 - 1),
            "the update the invulnerability ran out on is not the one the bullet killed on",
        );
    });
}
