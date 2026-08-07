//! **`--clear`: a run to the ending in a minute, on nothing but the shot key held.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine.
//!
//! What it takes to un-stub them: the fake 紅魔郷 has one stage and no way through six. It needs the
//! stages to follow one another, something able to hit the player for the invulnerability to be about,
//! and the result screen a clear ends on.
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well, so it is also
//! how the ending, the replay screen and the score file's write on the way out were all reached. It
//! fixes the mode rather than asking — `mode: pointdevice to start with; nobody is asked`.

/// Six stages with nothing able to hit the player, and no death anywhere in them.
///
/// Measured: `--clear` cleared stages 1 to 6 in **50.7 seconds** of wall clock — the log's stage starts
/// 5.3, 6.1, 7.7, 9.2 and 9.3 seconds apart — with `deaths=0` from the first stage to the ending and not
/// one `died in chapter` line, on nothing but the shot key being held. A later run of the same thing put
/// stages 1 to 6 in 35 seconds and went on into the ending.
#[test]
#[ignore = "the fake 紅魔郷 has one stage and no run through six"]
fn a_cleared_run_reaches_the_ending_with_no_death_in_it() {
    todo!("run six stages under --clear and assert deaths=0 and no died-in-chapter line")
}

/// The frames of invulnerability go back with the player's state, and the log said why they have to.
///
/// With the frames left where the last respawn had put them, `died in chapter 1` came 235ms after
/// `stage 1 chapter 1 (stage start)`, and again after each of the two retries. `Player::OnUpdate` runs at
/// chain priority **7** and the bullets are checked at **11**, so the state expired before the hit test
/// in the update it was written for. Writing the frames left with it fixed that.
#[test]
#[ignore = "the fake 紅魔郷 has nothing able to hit the player"]
fn the_invulnerability_outlasts_the_hit_test_in_the_update_it_was_written_for() {
    todo!(
        "put a bullet on the player at chain priority 11 with the state written at 7, and assert no \
         death"
    )
}

/// Nothing is written to any score file, and the game's own teardown still runs.
///
/// Measured over a clear that reached the result screen: the read went through — `score:
/// orb_score.dat opened in place of the game's own`, 47ms into scene 7 — and the write was refused,
/// `score: orb_score.dat not written, this run had nothing able to hit the player`. The game carried on
/// past it, scene 7 to 1 to 4, which is `WriteDataToFile` checking its open and its caller dropping the
/// answer. `score.dat` came out with the md5 and timestamp it went in with,
/// `004c8eda5a29a4ff985529838c21efe5` and `2026-07-29_22:44:45`, and orb's own with
/// `eca4048d984295dc91ca4f55050a779a` and `2026-08-01_21:18:05`.
///
/// Measured when orb's file was `orb_score.dat` and the fork was the `own_score_file` key. It is
/// `pointdevice_score.dat` now and the fork follows the mode chosen in the game; the seam and the
/// comparison are the same code.
#[test]
#[ignore = "the fake 紅魔郷 has no result screen for a clear to end on"]
fn a_cleared_run_writes_no_score_and_leaves_the_games_teardown_in_the_path() {
    todo!(
        "reach the result screen under --clear and assert the read happened and the write did not"
    )
}

/// No replay is offered, and the screen that saves one is skipped rather than the write refused.
///
/// Measured over the same `--clear` run to the ending:
///
/// ```text
/// f5966 scene=7                                                     the result screen
/// score: pointdevice_score.dat opened in place of the game's own     read as it was built
/// result: no replay is offered for a run with chapters
/// score: nothing written, this run had nothing able to hit the player
/// f6545 scene=1                                                     the title menu
/// score: pointdevice_score.dat opened in place of the game's own     the menu reading it again
/// ```
///
/// The result screen was up for **9.5 seconds** between those two scene lines, which is the high-score
/// name entry and the stats screen being played through as they always were — the state is written after
/// those, not instead of them. Then the title menu, with no save-replay screen in between.
#[test]
#[ignore = "the fake 紅魔郷 has no result screen and no replay screen after it"]
fn a_run_with_chapters_is_not_offered_the_screen_that_saves_a_replay() {
    todo!("reach the result screen with chapters kept and assert the replay screen never comes up")
}
