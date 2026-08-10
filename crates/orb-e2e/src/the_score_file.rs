//! **The score file forked per mode: which file each read lands in, and what a missing one costs.**
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
//! `pscr` and parses nothing else. `GameManager::AddedCallback` (0x41bcdc, once per **run** — the read is
//! inside the branch it takes only when it is not reinitialising, so a stage transition makes none; see
//! `a_stage_transition.rs`) and the ranking screen's added callback (0x42f47f) read all four
//! chunks, ranking and captures included. The write has one caller in the whole exe, 0x42f5cd in that
//! screen's deleted callback.

use crate::fake::th06::{
    CARD, CARD_NAME, CARD_STARTS, Fake, NOT_TRIED, Open, UNNAMED_CARD, the_run,
};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen, attempts_in, item};
use orb_core::mode::Mode;
use orb_sim::keys;

/// The two names an open of the score file can land in: the one the game asks for, and orb's own beside
/// it, named for the mode whose runs are in it.
const THEIRS: &str = "score.dat";
const OURS: &str = "pointdevice_score.dat";

/// The item of the title menu the score file's `clrd` chunk decides, read off the screen because a menu
/// is what no log can see — see `TITLE_MENU` for the eight the game draws.
const EXTRA_START: &str = "Extra Start";

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
/// Both halves of the bracket on `MainMenu::AddedCallback`, whose prologue at **0x43a464** is the six bytes
/// `push ebp; mov ebp,esp; sub esp,0x10`: inside it the fork is off, so the front end's read lands in the
/// game's own file and orb says nothing about it, and outside it every other open follows the mode. Neither
/// file is written by any of that.
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
/// The deleted callback writes whether a score was entered or the ranking was only looked at, so orb's file
/// appears with an empty record rather than waiting for a clear — no run has to be finished for one to exist.
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
/// Answering pointdevice at the *Score* item sends both the read and the write to orb's file, and answering
/// the game's own ranking next reads and writes `score.dat`: each file is touched only for its own answer.
/// Every item that opens the file asks first, which is what makes one session reaching both safe.
#[test]
fn each_modes_ranking_screen_writes_its_own_file_in_one_session() {
    in_its_own_process(|| {
        let game = launched("the-score-file-both-modes");

        // The screen each answer opens, read as its own two opens of the file: the screen's read as it is
        // built and its write as it goes down. The front end's own read when the menu is rebuilt afterwards
        // is the game's file either way and is not part of this — see the e2e test above, which is what says
        // so — so the opens are forgotten between the two halves of each answer.
        let opens_of = |mode: Mode| {
            game.forget_score_file_opens();
            asks_for_the_ranking(&game, mode);
            let (reads, _) = reads_and_writes(&game.score_file_opens());
            game.forget_score_file_opens();
            leaves_the_ranking(&game);
            let (_, writes) = reads_and_writes(&game.score_file_opens());
            (reads, writes)
        };

        let (reads, writes) = opens_of(Mode::Pointdevice);
        assert_eq!(reads, vec![OURS.to_owned()]);
        assert_eq!(
            writes,
            vec![OURS.to_owned()],
            "the pointdevice ranking wrote something other than orb's file, once",
        );

        // And the game's own ranking next, in the same launch: the mode the first answer left behind is not
        // where the second one lands, because every item that opens the file asks first.
        let (reads, writes) = opens_of(Mode::Normal);
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

/// A missing `pointdevice_score.dat` locks the unlocks rather than leaving them as they were, and the
/// front end's own read is what keeps them.
///
/// With no such file and `score.dat` beside it untouched, `Extra Start` comes up **locked** on the title
/// menu with pointdevice chosen: the menu is lit from the mode's file, and a mode whose file is new has
/// nothing in it. Which is what the bracket over `MainMenu::AddedCallback` is for.
///
/// So a failed read is not a no-op: `clrd`'s parse at **0x42b502** clears its destination before it looks
/// for the chunk — four records memset at **0x42b535**.
///
/// Both halves are here, because the second is what makes the first worth anything: the read that
/// *follows the mode* still lands in the file that is not there and still clears what the menu is lit
/// from — once a run, `GameManager::AddedCallback` — and what puts it back is the front end's own
/// read of the game's own file.
#[test]
fn a_missing_pointdevice_score_file_locks_the_unlocks() {
    in_its_own_process(|| {
        let game = Fake::attach("the-score-file-the-unlocks", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // The items themselves, which is what this one reads and no other e2e test does.
        game.draws_its_title_menu();
        game.at_the_title_menu();
        // Not there, which is what a first pointdevice session is looking at: the file arrives when a
        // ranking screen writes one, and nothing seeds it from the game's own.
        assert!(
            game.score_file(OURS).is_none(),
            "orb's file was there before any ranking screen had written it",
        );
        // And the menu is lit all the same, `Extra Start` among its items: the read that lights them is
        // the game's own file whichever mode orb is in.
        game.one_frame();
        assert_eq!(
            game.says(EXTRA_START).len(),
            1,
            "the front end offered no Extra Start with the game's own file there to light it from",
        );

        // The run, which is where the read that follows the mode happens: once at its start, and the file
        // it asks for is not there.
        game.in_a_pointdevice_run();
        assert!(
            game.log()
                .said("score: pointdevice_score.dat opened in place of the game's own, read"),
            "the stage's own read did not follow the mode:\n  {}",
            game.log().lines().join("\n  ")
        );
        // What that failed read cost: the destination is cleared before the chunk is looked for, so what
        // the menu would be lit from is gone — nothing was *left as it was*. What it is left with is not
        // zeros either, which is `ParseClrd`'s own fixup: a record with the magic, the version and every
        // clear count at 1, and 1 is not the 99 the Extra is behind.
        assert_eq!(
            game.image().unlocks(),
            fixed_up_by_a_failed_read(),
            "the read that failed left something other than its own memset and fixup",
        );
        assert!(
            !game.image().has_reached_max_clears(0, 0),
            "a record whose clear counts are the failed read's own says the game has been cleared",
        );

        // And the front end's own read putting them back, which is the whole of what the bracket buys: the
        // run ends, the menu is built, and the item is on it again.
        game.gives_the_run_up_at_its_own_pause();
        game.frames_until("the title menu after the run", 300, || {
            game.image().scene() == Scene::FrontEnd
                && game.image().front_end_now().screen == Screen::Title
        });
        game.one_frame();
        assert_eq!(
            game.says(EXTRA_START).len(),
            1,
            "the menu after the run was lit from the mode's own file, which has nothing in it",
        );
    });
}

/// What a `clrd` parse over a file that is not there leaves: the memset, and then the fixup.
///
/// Written out here rather than asked of the game, because asking the game would be asking the thing under
/// test. `ParseClrd` at 0x42b502 memsets each of the four records and then writes the magic, both lengths,
/// the version, the shot the record is about, and **1** into every one of the ten clear counts — which is a
/// record that looks like a record and says nobody has cleared anything.
fn fixed_up_by_a_failed_read() -> Vec<u8> {
    let mut records = Vec::new();
    for shot in 0..4u8 {
        let mut record = vec![0u8; 0x18];
        record[..4].copy_from_slice(b"CLRD");
        record[4..6].copy_from_slice(&0x18u16.to_le_bytes());
        record[6..8].copy_from_slice(&0x18u16.to_le_bytes());
        record[8] = 16;
        record[0x16] = shot;
        record[0xc..0x16].fill(1);
        records.extend(record);
    }
    records
}

/// A launch with no score file at all offers no Extra, and the record it is left with is the failed read's
/// own rather than zeros.
///
/// Which is the first launch of a fresh installation, and the one case where the front end's *own* read —
/// the game's file whichever mode orb is in — has nothing to find either. So the whole of what the menu
/// could be lit from is what `ParseClrd`'s fixup leaves: every clear count at 1.
///
/// **Both halves, because the outcome alone would pass on the wrong mechanism.** A gate on "the record holds
/// anything at all" would light the Extra from a fixup that says nobody has cleared anything; a gate on the
/// 99 `HasReachedMaxClears` compares against does not. So the record is read as well as the menu.
#[test]
fn a_launch_with_no_score_file_offers_no_extra_and_is_left_the_failed_reads_own_record() {
    in_its_own_process(|| {
        let game = Fake::attach("the-score-file-a-first-launch", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.draws_its_title_menu();
        // Nothing on any disk, which is what a fresh installation is. Before any frame, because the front
        // end is built on the launch's first one and its read is inside that build.
        game.has_no_score_file(THEIRS);
        game.at_the_title_menu();
        game.one_frame();

        assert_eq!(
            game.says(EXTRA_START).len(),
            0,
            "the front end offered an Extra Start with no score file to light it from",
        );
        assert_eq!(
            game.image().unlocks(),
            fixed_up_by_a_failed_read(),
            "the read that found nothing left something other than its own memset and fixup",
        );
        // And the item is off the menu for the reason the game has and not for want of a record: what is
        // there is a whole one, and every clear count in it is 1.
        assert!(
            !game.image().has_reached_max_clears(0, 0),
            "a record whose clear counts are the failed read's own says the game has been cleared",
        );
    });
}

/// What a session counted about spell cards survives a run that ended anywhere but the result screen.
///
/// For a run ended elsewhere the game's own ranking is built and taken down, that being where it writes.
/// Both modes, each into its own file, and both ways out of a run that are not the result screen — orb's
/// retry menu and the game's own `ESC`.
///
/// **Four attempts at this before it worked**, and the shape of each mistake is why the walk is what it
/// is: writing `curState` — the game's *result* — instead of asking the way the game asks left the front
/// end white twice and doubled once; taking `curState == 6` for the ranking being up caught a frame
/// mid-transition; guarding the two loops on `CHAIN_EXIT_SUCCESS`, which is 0 and so also an ordinary
/// chain answer, ended them after one update; and the front end's cursor was written into the menu object
/// being discarded rather than the one built on the way back. `ResultScreen.cpp:1527-1535` is how that
/// screen leaves, `MainMenu.cpp:848` the request being a reservation the front end acts on 60 frames later.
///
/// The three ways out a run has that are not the result screen, each in the mode it can be reached in: two
/// in 完全無欠モード, where orb's own menu is one of them, and the game's own in レガシーモード, where it is
/// the only one. The fourth way — a run that finished, whose own screen writes for it and which needs no
/// ranking built at all — is `the_screen_a_finished_run_ends_at.rs`'s.
#[test]
fn a_run_ended_away_from_the_result_screen_has_a_ranking_built_to_write() {
    in_its_own_process(|| {
        // ── タイトルに戻る, the retry menu's third item, which is the way out orb's own menu has. The
        // count it leaves is one only orb can have made: the game counts an attempt where a card
        // *starts*, and a chapter that begins inside one never starts it.
        {
            let game = launched("the-score-file-out-through-orbs-menu");
            game.in_a_pointdevice_run();
            at_the_cards_chapter(&game);
            retries_the_chapter(&game);
            assert_eq!(
                game.image().card_attempts(CARD),
                2,
                "the retry was not counted against the card",
            );
            // The run goes on to the chapter after, and the death that ends it is in that one: a death on
            // the very frame a chapter was put back on is not one orb notices, the frame it froze on
            // having been dropped along with everything else about it.
            game.frames_until("the chapter after the card", 900, || {
                game.log().said("chapter 4")
            });
            gives_the_run_up_at_orbs_menu(&game);
            wrote_what_the_run_counted(&game, OURS, 2);
            // And the ranking was asked for at the menu rather than by the run's own end: the choice is
            // where the flag goes in, and the frame it was acted on is one orb has deliberately dropped
            // everything it knew about — so there is no run of the frame before to notice ending.
            assert!(
                !game.log().said(WAITS_FOR_THE_RANKING),
                "a run given up at orb's own menu was noticed ending as well:\n  {}",
                game.log().lines().join("\n  ")
            );
        }

        // ── `esc` and then やめる, which is the game's own way out and the one a run in either mode has.
        // Here the run's own end is what asks for the ranking, orb having nothing to do with the way out.
        {
            let game = launched("the-score-file-out-through-the-games-own");
            game.in_a_pointdevice_run();
            at_the_cards_chapter(&game);
            game.gives_the_run_up_at_its_own_pause();
            wrote_what_the_run_counted(&game, OURS, 1);
            assert!(
                game.log().said(WAITS_FOR_THE_RANKING),
                "the run's own end is not what asked for the ranking:\n  {}",
                game.log().lines().join("\n  ")
            );
        }

        // ── And the same run in レガシーモード, whose file is the game's own: nothing of orb's is kept in
        // one, and what it counted about spell cards is still a session's work to lose.
        {
            let game = launched("the-score-file-out-of-a-legacy-run");
            game.in_a_legacy_run();
            game.frames_until("the card its boss puts up", 900, || {
                game.state().stage_frames > CARD_STARTS
            });
            game.gives_the_run_up_at_its_own_pause();
            wrote_what_the_run_counted(&game, THEIRS, 1);
            assert!(
                game.log().said(WAITS_FOR_THE_RANKING),
                "the run's own end is not what asked for the ranking:\n  {}",
                game.log().lines().join("\n  ")
            );
        }
    });
}

/// A ranking that never comes up writes nothing, has its request undone, and leaves the front end where it
/// was.
///
/// The one way that path is reached now: the ranking is asked for at the front end and nowhere else, so a
/// game that does not build the screen inside the 240 updates it is allowed is what is left of it. Three
/// things have to be true of it, and each was a defect once —
///
/// - **nothing is written**, the record in memory not having been put back;
/// - **the request is undone**, `MainMenu.gameState` going back to the item list rather than being left
///   holding the *Score* item: left there it is a reservation the front end acts on 60 frames later, which
///   is a ranking that comes up by itself, and the next run's end acting on it again;
/// - **and no screen's state is written**, there being no ranking to send away. That third one is the reason
///   the write is asked about at all and is the one this cannot read back: the address orb has is whichever
///   `ResultScreen` was built last and nothing clears it, so the write lands in a screen the game has already
///   freed, which a laid-out game has no way to tell from one it has not. What it *could* land in until orb
///   stopped asking for a ranking at a run's own result screen was the name entry somebody was standing at —
///   see `the_screen_a_finished_run_ends_at.rs`, which is where that half is read back.
#[test]
fn a_ranking_that_never_comes_up_writes_nothing_and_has_its_request_undone() {
    in_its_own_process(|| {
        let game = launched("the-score-file-a-ranking-that-never-comes-up");
        game.in_a_pointdevice_run();
        at_the_cards_chapter(&game);
        game.never_builds_the_ranking_it_is_asked_for();
        game.forget_score_file_opens();
        game.gives_the_run_up_at_its_own_pause();
        game.frames_until("the ranking given up on", 300, || {
            game.log().said("score: the ranking was not built after")
        });

        // Nothing written, and nothing read either: no screen came up to read one.
        let (reads, writes) = reads_and_writes(&game.score_file_opens());
        assert!(
            writes.is_empty(),
            "a ranking that never came up wrote {writes:?}",
        );
        assert_eq!(
            reads,
            vec![THEIRS.to_owned()],
            "the opens after it are not the front end's own read of the game's file coming back",
        );

        // And the front end is on its own item list, with no ranking coming up by itself behind it: the
        // request is a reservation, so one left behind is a screen that arrives a second later.
        game.frames_until("the title menu", 300, || {
            game.image().front_end_now().screen == Screen::Title
        });
        game.frames(120);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Title,
            "a ranking nobody asked for came up after the one orb asked for was given up on",
        );
        assert_eq!(
            game.image().scene(),
            Scene::FrontEnd,
            "the scene afterwards is not the front end's own",
        );
    });
}

/// **A chapter is an attempt at a spell card only where one is up, and orb counts none anywhere else.**
///
/// The card orb's own count goes against comes out of `ds:0x5a5f98`, which holds the last card a boss was on
/// and which **nothing clears** — not the card ending, not the stage, not the run. So a chapter with no card
/// in it reads whichever card came before, and the nonspell that follows a spell reads the spell it follows:
/// retried there, it would count an attempt at a card nobody was fighting. Which card is up is asked of
/// `g_EnemyManager.spellcardIsActive` instead, and asked *after* the snapshot is back — a spell chapter's
/// snapshot was taken at the card's own start, so what the restore puts the game back into is the card.
///
/// The other half is what a count against a card nobody has fought puts on the screen.
/// `GameManager::AddedCallback` fills all 64 records from `Rng::GetRandomU16` at every run's start and writes
/// back only the magic, the two lengths, the version, the card's number and the two counts — see
/// `CARD_HISTORY` — so a card the file holds no record of stands in memory carrying a name nobody wrote, and
/// the ranking screen draws that name the moment its attempts are not zero (0x42e26e).
///
/// What that puts on the ranking is a row drawing the generator's own bytes where the card's name belongs,
/// against no captures and however many attempts orb added. **A retry cannot heal a row already written
/// wrong**: a chapter's snapshot is taken after the name was copied in, so putting it back never starts the
/// card again. Only the card starting for real does — the sum at 0x4097e8 then disagrees and both counts go —
/// or a run picked up, whose playback starts it and whose landing keeps the name that start wrote: see
/// [`a_run_picked_up_keeps_the_name_its_playback_learned`].
#[test]
fn only_a_chapter_with_a_spell_card_up_counts_an_attempt() {
    in_its_own_process(|| {
        let game = launched("the-score-file-which-chapters-count");
        game.in_a_pointdevice_run();
        // The record as the run's own start left it, which is all three of the things the fill leaves and a
        // block nothing had filled would have none of: the magic on it, nothing tried, and a name of its own
        // that came out of the generator rather than out of a card.
        assert!(
            game.image()
                .card_records()
                .iter()
                .any(|(card, attempts)| *card == UNNAMED_CARD && *attempts == 0),
            "the run's start left no record at all for the card nobody has named",
        );
        assert!(
            game.image()
                .card_name(UNNAMED_CARD)
                .is_some_and(|name| !name.is_empty() && name != CARD_NAME),
            "the record holds no name of its own for a count to put on the screen",
        );

        // ── The chapter the stage's own start is, retried before any card of the run has started: nothing to
        // count against, and `ds:0x5a5f98` still holding the 0 a fresh process leaves.
        retries_the_chapter(&game);
        assert_eq!(
            game.image().card_attempts(UNNAMED_CARD),
            0,
            "an attempt was counted where no spell card was up",
        );
        assert!(
            game.log().said(NO_CARD_UP),
            "the count refused is not in the log:\n  {}",
            game.log().lines().join("\n  ")
        );

        // ── The card's own chapter, where the game counted the start and orb counts the retry.
        at_the_cards_chapter(&game);
        retries_the_chapter(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the retry of the card's own chapter was not counted against it",
        );

        // ── And the nonspell after it, which reads that same card out of `ds:0x5a5f98` and is an attempt at
        // none of it: the card is over and what is being fought has no name.
        game.frames_until("the chapter after the card", 900, || {
            game.log().said("chapter 4")
        });
        retries_the_chapter(&game);
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the nonspell after the card counted an attempt at the card it follows",
        );

        // ── And what the file the session writes holds, which is what the next session reads back.
        game.frames(A_CHAPTER_TO_SETTLE);
        gives_the_run_up_at_orbs_menu(&game);
        wrote_what_the_run_counted(&game, OURS, 2);
        assert_eq!(
            attempts_in(
                &game.score_file(OURS).expect("the file the ranking wrote"),
                UNNAMED_CARD
            ),
            0,
            "the file the ranking wrote counts an attempt at a card nobody fought",
        );

        // ── And the two rows off the screen the session looks at: the card's own name against its count, and
        // the five question marks a card nobody has tried gets rather than the bytes they stand in front of.
        asks_for_the_ranking(&game, Mode::Pointdevice);
        game.forget();
        game.frame();
        let fought = game.says(&format!("CARD {CARD}"));
        let untried = game.says(&format!("CARD {UNNAMED_CARD}"));
        assert_eq!(
            (fought.len(), untried.len()),
            (1, 1),
            "the ranking is not one row apiece for the card fought and the card nobody named",
        );
        assert!(
            game.says(CARD_NAME)
                .iter()
                .any(|written| written.y == fought[0].y),
            "the card's own name is not what its row shows",
        );
        assert!(
            game.says(NOT_TRIED)
                .iter()
                .any(|mark| mark.y == untried[0].y),
            "the row of a card the game has not named shows something other than {NOT_TRIED}",
        );
    });
}

/// What orb says where a chapter is retried with no card up, which is every chapter but a spellcard's.
const NO_CARD_UP: &str = "score: no spell card is up; no attempt counted";

/// **A run picked up keeps the name its playback learned, and its landing is counted.**
///
/// The playback starts every card the run had passed, so the game names each of their records on the way
/// through (0x409720) and counts an attempt at each (0x409824). The counts are put back as they were before
/// the buttons went in — a run picked up would otherwise arrive having counted every card it passed — and
/// the names are not, a name being what the playback *learned* rather than what it counted. Putting those
/// back too leaves the record carrying the fill's own bytes, which is a card orb then refuses to count
/// against and a row the ranking screen never draws a name for again: the landing is *inside* the card, so
/// nothing starts it a second time.
///
/// **What putting the names back too costs** is the other half of that, and it is what this walk is shaped
/// to reach: a run picked up into a card's own chapter, given up, and the ranking asked for at the title
/// menu's `Score`. With the names put back, orb refuses to count against a card the game has no name for and
/// the row keeps its 「？？？？？」 for good; with them left as the playback wrote them, the attempt is
/// counted and the name is legible on the row.
#[test]
fn a_run_picked_up_keeps_the_name_its_playback_learned() {
    in_its_own_process(|| {
        let game = launched("the-score-file-a-run-picked-up");
        game.in_a_pointdevice_run();
        at_the_cards_chapter(&game);
        gives_the_run_up_at_orbs_menu(&game);
        wrote_what_the_run_counted(&game, OURS, 1);

        // And orb's file moved aside, which is what leaves the card's record to the fill: the run is still
        // written down under `pointdevice_resume/` and is picked up from there, while the file the run-start
        // read would have named the card out of is not there to be read.
        game.has_no_score_file(OURS);
        game.picks_the_run_up();

        // The name the playback wrote, which is the card's own and not the fill's.
        assert_eq!(
            game.image().card_name(CARD).as_deref(),
            Some(CARD_NAME),
            "the landing put back the name the playback learned along with the count",
        );
        // And the landing counted, this being an attempt at that chapter the same way a retry is — the one
        // the playback made on its way there having been taken back off.
        assert_eq!(
            game.image().card_attempts(CARD),
            1,
            "the landing counted no attempt, or the playback's own count stayed",
        );
        assert!(
            game.log().said("resume: attempt 1 at this spell card"),
            "the landing's attempt is not in the log:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And then the walk this was found by, to the end: the run picked up is played on, given up, and its
        // ranking asked for. What a session sees is the row, and the row is the whole of what it can see —
        // the record in memory above is what the row is drawn from, and neither says anything on its own
        // about what somebody looking at the screen is told.
        game.frames(A_CHAPTER_TO_SETTLE);
        gives_the_run_up_at_orbs_menu(&game);
        wrote_what_the_run_counted(&game, OURS, 1);
        asks_for_the_ranking(&game, Mode::Pointdevice);
        game.forget();
        game.frame();
        let row = game.says(&format!("CARD {CARD}"));
        assert_eq!(
            row.len(),
            1,
            "the ranking is not one row for the card the run was picked up into",
        );
        assert!(
            game.says(CARD_NAME)
                .iter()
                .any(|written| written.y == row[0].y),
            "the card's own name is not what its row shows",
        );
        assert!(
            game.says(NOT_TRIED).iter().all(|mark| mark.y != row[0].y),
            "a card with an attempt against it still shows {NOT_TRIED}",
        );
    });
}

/// Frames between a chapter put back and the death that ends the run, so that the death is one orb
/// notices: the frame a restore lands on is one it has dropped everything it knew about.
const A_CHAPTER_TO_SETTLE: u32 = 30;

/// What orb says where a run's own end is what asked for the ranking, which is every way out of one but
/// orb's own menu: that one sets the flag at the choice, on a frame whose run orb has already stopped
/// comparing anything against.
const WAITS_FOR_THE_RANKING: &str =
    "score: a run ended; what it counted waits for the ranking to be built and taken down";

/// The run played as far as the chapter its boss's spell card is, which is where the game counts an
/// attempt at that card: a chapter beginning inside one never starts it, so this is the number orb's own
/// count is added to.
fn at_the_cards_chapter(game: &Fake) {
    let log = game.log();
    game.frames_until("the chapter the card is", 900, || {
        log.said(&format!(
            "at frame {CARD_STARTS}, {CARD_STARTS} frame(s) of buttons"
        ))
    });
    assert_eq!(
        game.image().card_attempts(CARD),
        1,
        "the game counted no attempt where the card started",
    );
}

/// 被弾 and then チャプターをやり直す, which is the retry menu's first item and the attempt orb counts.
fn retries_the_chapter(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.hit();
    game.frame();
    assert!(
        log.said_since(from, "died in chapter"),
        "the death was not noticed:\n  {}",
        log.lines().join("\n  ")
    );
    game.press_until(keys::Z, "the retry menu answered", || {
        log.said_since(from, "retry: the chapter again on the keyboard")
    });
}

/// 被弾 and then タイトルに戻る, which is up once from the item the cursor starts on, and the question
/// that item asks, whose own cursor starts on いいえ.
///
/// The frames each holds its keys off for are waited out first: a direction pressed inside those is one
/// nothing moved on — see `READS_KEYS_AFTER`.
fn gives_the_run_up_at_orbs_menu(game: &Fake) {
    let log = game.log();
    // Where this walk begins, so that a session giving a second run up waits for its own lines: a log asked
    // whether it has *ever* said something is answered yes by the run before, and nothing is waited out.
    let from = log.written();
    game.hit();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said_since(from, "retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said_since(from, "retry: the run is given up")
    });
}

/// The ranking built and taken down after it, and the file that wrote: which name the write landed in, and
/// the count in it.
///
/// The opens are forgotten first, so what is read back is that screen's own: the reads the run itself made
/// are behind it, and the front end's read when the menu comes back is the game's own file either way.
///
/// And the write is what the wait is on, rather than the line orb says on its way out of the screen: a
/// session with two runs in it says that line twice, and a log asked whether it has *ever* said something is
/// answered yes by the first of them before the second screen has been asked for.
fn wrote_what_the_run_counted(game: &Fake, name: &str, attempts: u16) {
    game.forget_score_file_opens();
    game.frames_until("the ranking built and taken down", 300, || {
        !reads_and_writes(&game.score_file_opens()).1.is_empty()
    });
    let (_, writes) = reads_and_writes(&game.score_file_opens());
    assert_eq!(
        writes,
        vec![name.to_owned()],
        "the write landed somewhere other than the file this run's mode is in, or happened twice",
    );
    // And what is in it: the record as the memory held it when the screen went down, which is this
    // session's count and not the one the file was read with.
    let written = game.score_file(name).expect("the file that screen wrote");
    assert_eq!(
        attempts_in(&written, CARD),
        attempts,
        "the file that screen wrote holds a count this session did not make",
    );
    assert_eq!(
        game.image().card_attempts(CARD),
        attempts,
        "the record in memory was left holding something else",
    );
}
