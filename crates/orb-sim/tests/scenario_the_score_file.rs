//! **The score file forked per mode: which file each read lands in, and what a missing one costs.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! **The game opens the file and orb decides which one it gets.** The fork is a hook over the exe's
//! `CreateFileA`, reached in a real launch by patching its import table and here by the game handing its own
//! over — `Originals::create_file`, the same answer `create_window` is, and see
//! [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md). What
//! crosses that call is the path and the access, which is the whole of what these read back: no file is on
//! any disk. The fork's own arithmetic is covered — `score.rs`'s
//! `the_game_score_file_is_forked_where_the_game_asked_for_it` and the six beside it — and what these add is
//! what the *game* does with the file it gets.
//!
//! **The three reads are told apart in the exe.** `MainMenu::AddedCallback` (0x43a5c0) is the only one the
//! front end's own items are lit from: it fills `g_GameManager` at 0x69ccd0 and 0x69cd30 with `clrd` and
//! `pscr` and parses nothing else. `GameManager::AddedCallback` (0x41bcdc, once per stage) and the ranking
//! screen's added callback (0x42f47f) read all four chunks, ranking and captures included. The write has
//! one caller in the whole exe, 0x42f5cd in that screen's deleted callback.

mod fake;

use fake::th06::{CARD, Fake, Open, the_run};
use fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen, item};
use orb_core::mode::Mode;
use orb_sim::keys;

/// The two names an open of the score file can land in: the one the game asks for, and orb's own beside
/// it, named for the mode whose runs are in it.
const THEIRS: &str = "score.dat";
const OURS: &str = "pointdevice_score.dat";

/// A launch at its title menu, with the front end's own read of the file already behind it.
fn launched(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    game
}

/// Walks the title menu's cursor to the item the ranking is behind, and answers orb's question about which
/// of the two rankings with `mode`.
///
/// The walk somebody makes: every item that opens the file asks first, which is what makes one session
/// reaching both files safe.
fn asks_for_the_ranking(game: &Fake, mode: Mode) {
    let log = game.log();
    game.frames_until("the title menu ready to act on a press", 120, || {
        let front = game.image().front_end_now();
        front.screen == Screen::Title && front.acts_on_a_press()
    });
    let cursor = game.image().front_end_now().cursor;
    for _ in cursor..item::SCORE {
        game.press(keys::DOWN);
    }
    for _ in item::SCORE..cursor {
        game.press(keys::UP);
    }
    assert_eq!(
        game.image().front_end_now().cursor,
        item::SCORE,
        "the cursor is not on the item the ranking is behind",
    );
    game.press(keys::Z);
    assert!(
        log.said("menu: Scores is under the cursor, asking which mode"),
        "the press did not put the ranking's question up:\n  {}",
        log.lines().join("\n  ")
    );
    // 完全無欠モード is the item the cursor starts on, so the game's own ranking is one press down. A
    // direction cannot be repeated the way a decide can — a list of two is on the other item every press —
    // so the frames the question holds its keys off for are waited out first.
    if mode == Mode::Normal {
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
    }
    game.press_until(keys::Z, "the ranking asked for", || {
        game.image().scene() == Scene::Ranking
    });
    // One frame past the one it was asked for on, which is where it is *built*: the supervisor acts on a
    // scene that has been asked for on the update after, and the read of the file is inside that build.
    game.frame();
}

/// Leaves the ranking screen, which is what writes the file.
fn leaves_the_ranking(game: &Fake) {
    game.press(keys::X);
    game.frames_until("the title menu again", 60, || {
        game.image().scene() == Scene::FrontEnd
            && game.image().front_end_now().screen == Screen::Title
    });
}

/// Every open of the score file that was for reading, and every one that was for writing.
fn reads_and_writes(opens: &[Open]) -> (Vec<String>, Vec<String>) {
    let of = |write: bool| {
        opens
            .iter()
            .filter(|open| open.write == write)
            .map(|open| open.path.clone())
            .collect()
    };
    (of(false), of(true))
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
fn the_front_ends_read_is_the_games_own_file_and_the_ranking_follows_the_mode() {
    in_its_own_process(|| {
        let game = launched("the-score-file-front-end");

        // The front end's own read, which is the one open whose answer is the game's own file whatever the
        // mode: `reading_unlocks` brackets exactly that callback and nothing else.
        let (reads, writes) = reads_and_writes(&game.score_file_opens());
        assert_eq!(
            reads,
            vec![THEIRS.to_owned()],
            "the front end's read did not land in the game's own file",
        );
        assert!(writes.is_empty(), "the front end's read wrote something");
        // And orb said nothing about it, which is what the bracket buys: a swapped open is a line, and this
        // one was not swapped.
        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("opened in place of the game's own")),
            "the front end's read was sent to orb's file:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the ranking, answered pointdevice: that one follows the mode.
        game.forget_score_file_opens();
        asks_for_the_ranking(&game, Mode::Pointdevice);
        let (reads, writes) = reads_and_writes(&game.score_file_opens());
        assert_eq!(
            reads,
            vec![OURS.to_owned()],
            "the ranking's read did not follow the mode it was answered with",
        );
        assert!(
            writes.is_empty(),
            "the ranking wrote the file on its way up rather than on its way down",
        );
        assert!(
            game.log()
                .said("score: pointdevice_score.dat opened in place of the game's own"),
            "orb did not say the open was swapped:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Leaving the ranking screen writes orb's file whether or not a score was entered.
///
/// Measured in the same session, without a run being finished: `pointdevice_score.dat` came out at
/// **4,224 bytes**, `4f733fc56b8e80d3a511acfc7ba8cb0d`, against `score.dat`'s 8,724. Leaving the ranking
/// screen is what wrote it — the deleted callback writes whether a score was entered or the ranking was
/// only looked at — so orb's file appears with an empty record rather than waiting for a clear.
#[test]
fn leaving_the_ranking_writes_orbs_file_with_nothing_entered() {
    in_its_own_process(|| {
        let game = launched("the-score-file-leaving-the-ranking");
        // Not there to begin with, which is what makes the write below the thing that made it: orb's file
        // is not started as a copy of the game's.
        assert!(
            game.score_file(OURS).is_none(),
            "orb's file was there before any ranking screen had written it",
        );

        asks_for_the_ranking(&game, Mode::Pointdevice);
        game.forget_score_file_opens();
        leaves_the_ranking(&game);

        let (_, writes) = reads_and_writes(&game.score_file_opens());
        assert_eq!(
            writes,
            vec![OURS.to_owned()],
            "leaving the ranking did not write orb's file",
        );
        assert!(
            game.score_file(OURS).is_some(),
            "orb's file is still not there after the screen that writes it went down",
        );
        // With nothing entered: no run was finished in this session at all, so what it holds is an empty
        // record rather than a score.
        assert_eq!(
            game.image().card_attempts(CARD),
            0,
            "a session that played nothing counted an attempt",
        );
    });
}

/// Each mode's ranking screen writes its own file, and the mode an answer left behind cannot reach the
/// other.
///
/// Measured in one session: answering pointdevice at the *Score* item sent the read to orb's file at
/// 357617468ms and the write at `2026-08-05_00:21:53`; answering the game's own ranking next — `mode:
/// normal, was pointdevice` at 357638484ms — read and wrote `score.dat`, mtime `2026-08-05_00:22:28`.
/// Every item that opens the file asks first, which is what makes that safe.
#[test]
fn each_modes_ranking_screen_writes_its_own_file_in_one_session() {
    in_its_own_process(|| {
        let game = launched("the-score-file-both-modes");

        // The trip each answer makes, read as its own two opens: the screen's read on the way up and its
        // write on the way down. The front end's own read when the menu is rebuilt afterwards is the game's
        // file either way and is not part of this — see the scenario above, which is what says so — so the
        // opens are forgotten between the two halves of each trip.
        let trip = |mode: Mode| {
            game.forget_score_file_opens();
            asks_for_the_ranking(&game, mode);
            let (reads, _) = reads_and_writes(&game.score_file_opens());
            game.forget_score_file_opens();
            leaves_the_ranking(&game);
            let (_, writes) = reads_and_writes(&game.score_file_opens());
            (reads, writes)
        };

        let (reads, writes) = trip(Mode::Pointdevice);
        assert_eq!(reads, vec![OURS.to_owned()]);
        assert_eq!(
            writes,
            vec![OURS.to_owned()],
            "the pointdevice ranking wrote something other than orb's file, once",
        );

        // And the game's own ranking next, in the same launch: the mode the first answer left behind is not
        // where the second one lands, because every item that opens the file asks first.
        let (reads, writes) = trip(Mode::Normal);
        assert!(
            game.log().said("mode: normal, was pointdevice"),
            "the second answer did not change the mode:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            reads,
            vec![THEIRS.to_owned()],
            "the game's own ranking read orb's file",
        );
        assert_eq!(
            writes,
            vec![THEIRS.to_owned()],
            "the game's own ranking wrote orb's file, or wrote its own more than once",
        );

        // Each file written once and no more, over the session as a whole: two answers, two writes.
        assert!(game.score_file(OURS).is_some());
        assert!(game.score_file(THEIRS).is_some());
    });
}

/// A missing `pointdevice_score.dat` locks the unlocks rather than leaving them as they were.
///
/// A stub: `#[ignore]`d, and `todo!()` where the assertion goes.
///
/// Measured on a launch with no such file — `score.dat` beside it untouched, mtime
/// `2026-08-02_18:47:50`: `Extra Start` came up **locked** on the title menu with pointdevice chosen, and
/// the log's only score line for that menu was `score: pointdevice_score.dat opened in place of the
/// game's own` at 338359890ms.
///
/// So a failed read is not a no-op: `clrd`'s parse at **0x42b502** clears its destination before it looks
/// for the chunk — four records memset at **0x42b535**.
///
/// What it takes to un-stub it: the failed open is reachable now — orb's file is not there until a ranking
/// screen has written it, and `Fake::reads_the_score_file` puts an empty record back where one fails, which
/// is what 0x42b535 does. What is not reachable is the half the assertion is about: which items the game's
/// own menu lights, and whether `Extra Start` is among them. The laid-out front end has eight items and no
/// unlocks in it, and *what the front end offers* is on the list of things the log cannot see — see
/// *And the front end's own answers* in [TODO.md](../../../TODO.md).
#[test]
#[ignore = "the laid-out front end has no unlocks for a failed read to lock"]
fn a_missing_pointdevice_score_file_locks_the_unlocks() {
    todo!("give the front end no pointdevice file and assert Extra Start comes up locked")
}

/// What a session counted about spell cards survives a run that ended anywhere but the result screen.
///
/// A stub: `#[ignore]`d, and `todo!()` where the assertion goes.
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
