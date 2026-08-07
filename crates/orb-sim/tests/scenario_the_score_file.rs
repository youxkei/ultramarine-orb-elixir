//! **The score file forked per mode: which file each read lands in, and what a missing one costs.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine.
//!
//! What it takes to un-stub them: the fork is an import hook on the exe's `CreateFileA`, and a scenario
//! cannot install one — `scenario_legacy_run.rs` says so where it stops short of asking which file a run's
//! score landed in. So these want the hook reachable through the seam the way `memtrack::regions` became
//! `orb_api::mem::game_regions`, and a laid-out 紅魔郷 with the ranking screen and the front end's own
//! read in it. The fork's own arithmetic is covered — `score.rs`'s
//! `the_game_score_file_is_forked_where_the_game_asked_for_it` and the six beside it — and what these add
//! is what the *game* does with the file it gets.
//!
//! **The three reads are told apart in the exe.** `MainMenu::AddedCallback` (0x43a5c0) is the only one the
//! front end's own items are lit from: it fills `g_GameManager` at 0x69ccd0 and 0x69cd30 with `clrd` and
//! `pscr` and parses nothing else. `GameManager::AddedCallback` (0x41bcdc, once per stage) and the ranking
//! screen's added callback (0x42f47f) read all four chunks, ranking and captures included. The write has
//! one caller in the whole exe, 0x42f5cd in that screen's deleted callback.

/// A missing `pointdevice_score.dat` locks the unlocks rather than leaving them as they were.
///
/// Measured on a launch with no such file — `score.dat` beside it untouched, mtime
/// `2026-08-02_18:47:50`: `Extra Start` came up **locked** on the title menu with pointdevice chosen, and
/// the log's only score line for that menu was `score: pointdevice_score.dat opened in place of the
/// game's own` at 338359890ms.
///
/// So a failed read is not a no-op: `clrd`'s parse at **0x42b502** clears its destination before it looks
/// for the chunk — four records memset at **0x42b535**.
#[test]
#[ignore = "the fork is an import hook on CreateFileA and a scenario cannot install one"]
fn a_missing_pointdevice_score_file_locks_the_unlocks() {
    todo!("give the front end no pointdevice file and assert Extra Start comes up locked")
}

/// Leaving the ranking screen writes orb's file whether or not a score was entered.
///
/// Measured in the same session, without a run being finished: `pointdevice_score.dat` came out at
/// **4,224 bytes**, `4f733fc56b8e80d3a511acfc7ba8cb0d`, against `score.dat`'s 8,724. Leaving the ranking
/// screen is what wrote it — the deleted callback writes whether a score was entered or the ranking was
/// only looked at — so orb's file appears with an empty record rather than waiting for a clear.
#[test]
#[ignore = "the fork is an import hook on CreateFileA and a scenario cannot install one"]
fn leaving_the_ranking_writes_orbs_file_with_nothing_entered() {
    todo!("walk the ranking screen with no score entered and assert orb's file was written")
}

/// The front end's own read is the game's file, and every other open follows the mode.
///
/// Measured over a session on 2026-08-05, both halves of the bracket on `MainMenu::AddedCallback`:
///
/// - `unlocks read hook installed, original at 0x02940000` at 357601218ms, so the six bytes at
///   **0x43a464** were the `push ebp; mov ebp,esp; sub esp,0x10` expected of it.
/// - The menu was up at 357605156ms (`f0 scene=1`) with **no `score:` line anywhere in between**, where
///   the same point in the session before the bracket had one at 338359890ms.
/// - Answering pointdevice at the *Score* item (357616406ms) and the screen coming up at 357617437ms
///   (`f555 scene=6`) was followed by `score: pointdevice_score.dat opened in place of the game's own`
///   31ms later.
/// - Neither file was written by any of that: mtimes stayed `2026-08-04_19:01:40` and
///   `2026-08-04_19:03:35`.
#[test]
#[ignore = "the fork is an import hook on CreateFileA and a scenario cannot install one"]
fn the_front_ends_read_is_the_games_own_file_and_the_ranking_follows_the_mode() {
    todo!(
        "bracket the front end's read and assert it lands in the game's file and the ranking in orb's"
    )
}

/// Each mode's ranking screen writes its own file, and the mode an answer left behind cannot reach the
/// other.
///
/// Measured in one session: answering pointdevice at the *Score* item sent the read to orb's file at
/// 357617468ms and the write at `2026-08-05_00:21:53`; answering the game's own ranking next — `mode:
/// normal, was pointdevice` at 357638484ms — read and wrote `score.dat`, mtime `2026-08-05_00:22:28`.
/// Every item that opens the file asks first, which is what makes that safe.
#[test]
#[ignore = "the fork is an import hook on CreateFileA and a scenario cannot install one"]
fn each_modes_ranking_screen_writes_its_own_file_in_one_session() {
    todo!("answer both modes at the Score item in one launch and assert each file was written once")
}

/// What a session counted about spell cards survives a run that ended anywhere but the result screen.
///
/// Watched on 2026-08-05 in play. A run ended elsewhere is taken through the game's own ranking, which is
/// where it writes: `score: a run ended; what it counted waits for the trip through the ranking` at
/// 371528265ms, `score.dat opened as the game's own, write` **297ms** later, and `score: taken through the
/// ranking in 84 update(s) — cur=1 wanted=1`. The same shape with `pointdevice_score.dat` for a
/// pointdevice run. Both modes, and both ways out of a run — orb's retry menu and the game's own `ESC`.
/// The counts read off the game's own screen afterwards were up: the attempt count against the card a
/// chapter was retried at, and the capture count for a card taken in a legacy run stopped partway.
///
/// **Four attempts at this before it worked**, and the shape of each mistake is why the walk is what it
/// is: writing `curState` — the game's *result* — instead of asking the way the game asks left the front
/// end white twice and doubled once; taking `curState == 6` for the ranking being up caught a frame
/// mid-transition; guarding the trip's loops on `CHAIN_EXIT_SUCCESS`, which is 0 and so also an ordinary
/// chain answer, ended them after one update; and the front end's cursor was written into the menu object
/// being discarded rather than the one built on the way back. `ResultScreen.cpp:1527-1535` is how that
/// screen leaves, `MainMenu.cpp:848` the request being a reservation the front end acts on 60 frames later.
#[test]
#[ignore = "the walk through the ranking needs the front end rebuilding itself, which the laid-out 紅魔郷 does not do"]
fn a_run_ended_away_from_the_result_screen_is_taken_through_the_ranking_to_write() {
    todo!(
        "end a run through the retry menu and assert the trip through the ranking wrote the counts"
    )
}
