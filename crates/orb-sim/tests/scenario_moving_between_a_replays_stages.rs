//! **A replay played out the same way after being moved between its stages, to the last digit.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine. This is what
//! `--collect` and `--judge` stand on: a pass that builds a midstage chapter table steps between
//! boundaries and moves between stages, so a replay that stops playing out the same way makes every
//! boundary it judged afterwards worthless.
//!
//! What it takes to un-stub them: the laid-out 紅魔郷 has no replay and no generator whose stream can be
//! counted. It needs the replay manager's record, `Rng::GetRandomU16` drawn from where the game draws it,
//! and the player's position moved by recorded input rather than by a scenario's presses.
//!
//! **How it was settled**: a line per update — the replay's input clock, the buttons it fed, the player's
//! position, and how many numbers the generator has given out — written for two passes over stage 1 and
//! compared. **9011 frames**, the whole stage including the boss, agreeing **to the last digit** across a
//! move out to stage 2 and back.

/// The run's recording is not ended by a stage teardown, so leaving a stage part way does not blank it.
///
/// The game ends the run's recording at every stage teardown, writing a blank input and a frame number no
/// run reaches into the record it holds — which during playback is the replay's own, at the entry playback
/// has reached. Leaving a stage part way therefore terminated it there, and playing it again the player
/// took no input from that frame on.
///
/// Measured before this was held back, in the log at **297414375ms**: out of stage 1 around script frame
/// 250 and straight back, **three lives gone by frame 1027**.
#[test]
#[ignore = "the laid-out 紅魔郷 has no replay record for a teardown to blank"]
fn a_stage_teardown_does_not_end_the_recording_being_played_back() {
    todo!("leave a stage mid-way and assert the record still holds the inputs past that frame")
}

/// Starting a replay at a stage puts the score and the extra lives paid for back to nothing first.
///
/// Starting at a stage is not the run carrying on. Measured before this: stage 1 began with the score the
/// run was left on — **7417420** at 303310937ms — crossed 10,000,000 at frame 3240 and took an extra life
/// the recording never had, which raised rank from **21 to 23**. The generator's stream shifted three
/// frames 173 frames later, and the player, moving on recorded inputs, was hit.
#[test]
#[ignore = "the laid-out 紅魔郷 has no replay to start at a stage"]
fn starting_a_replay_at_a_stage_puts_the_score_and_the_extra_lives_back_to_nothing() {
    todo!(
        "start a replay at stage 1 with a score behind it and assert the score and rank start clean"
    )
}

/// A screen shake does not outlive the stage that started it, because it draws from the generator.
///
/// A shake writes the play field's rectangle from **two numbers out of the generator every frame**, and
/// `Player::AddedCallback` measures where the player starts from that rectangle. Outliving the stage is
/// deliberate for the fade between two stages and wrong for a shake.
///
/// Measured: a bomb within the shake's **80 frames** of a stage move began the next stage with the player
/// at **`192.00,380.87`** where it starts one at **`192.00,384.00`**, and four numbers a frame going out
/// of the stream. Checked by leaving stage 2 while a bomb's shake was running — taken down at
/// **61041250ms** — and holding the stage 1 that followed against a menu-started pass of it: identical
/// from its first frame to the **742nd**, where the replay was stopped by hand.
#[test]
#[ignore = "the laid-out 紅魔郷 has no screen shake and no generator to draw from"]
fn a_screen_shake_does_not_reach_the_stage_after_the_one_that_started_it() {
    todo!(
        "bomb inside 80 frames of a stage move and assert the next stage starts the player at 384.00"
    )
}

/// The whole of it: two passes over one stage agreeing to the last digit across a move and back.
///
/// **9011 frames**, the whole stage including the boss, with the replay's input clock, the buttons it fed,
/// the player's position and the count of numbers the generator has given out identical on both sides.
/// This is the scenario the three above are the parts of, and the one `--collect` and `--judge` rest on.
#[test]
#[ignore = "the laid-out 紅魔郷 has no replay to play twice"]
fn a_stage_played_twice_across_a_move_agrees_to_the_last_digit() {
    todo!(
        "play stage 1 twice, once straight and once across a move to stage 2 and back, and diff them"
    )
}
