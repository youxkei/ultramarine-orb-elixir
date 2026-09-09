//! **Whether a run of th07's replay is kept, which is the setting and the question behind it.**
//!
//! On, every one of them is kept and nothing is asked. Off is not *no replays*: it is orb asking, which is
//! where somebody says which of their runs were worth keeping — so the answer is a question orb puts up, and
//! いいえ is what takes the file away.
//!
//! **Asked at the two moments nothing more will ever be added to one.** The screen a run ends at, which is
//! where the game's own offer would have been — written past, a replay of the game's own being the path that
//! survived with nothing in it of the attempts that did not — and はじめから, where the run left unfinished
//! is about to be written over. What is this game's own is where the second of those goes up: over th07's
//! shot select, which is the screen the question about the run left unfinished is asked over, and the press
//! that answers it is handed back to that screen's own decide.
//!
//! The question orb asks is its own screen, drawn in the face the machine lent — th07 ships no font — and
//! what it reads its keys with is what every question of orb's reads.

use crate::fake::th07::Fake;
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::Th07;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the run being played.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";

/// The name orb gives the run these play — Normal, and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";
/// The date the simulated host's calendar is set to, which a kept replay's name carries.
const DATED: &str = "20040815-214233";

/// Whose runs these are, as `orb.yaml` says.
const PLAYER: &str = "ゆうか";

/// How long a stretch of the fight is here, which is only the frames a chapter of one needs.
const APART: u32 = 120;

/// A launch that asks, which is what a file that says nothing about it gives: `save_replays` off.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = false;
            config.player_name = PLAYER.to_owned();
        },
    );
    game.streams_its_song(0);
    game
}

/// A run played as far as its midboss and then given up at the retry menu, which leaves both files behind —
/// the chapter to be picked up, and the replay to be asked about.
fn a_run_left_behind(game: &Fake) {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames(APART);
    game.the_boss_arrives();
    game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
    game.frames(APART);
    game.gives_the_run_up();
    game.frames_until_a_thread("the replay written down", || {
        orb_core::replay::read_being_played(game.dir(), SLOT, &Th07).is_some()
    });
}

/// The same run started again and はじめから answered, which is where the question about the replay of the
/// run being written over goes up.
fn started_over(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.at_the_shot_select();
    game.press_until(keys::Z, "the question about the run left behind", || {
        log.said_since(from, "asking where to start")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::DOWN);
    game.press_until(keys::Z, "はじめから answered", || {
        log.said_since(from, "resume: from the beginning, answered on the keyboard")
    });
    game.frames_until("the question about the replay", 60, || {
        log.said_since(
            from,
            &format!("replay: {SLOT} is being written over; asking whether to keep it"),
        )
    });
}

/// And the run the answer was given on the way into, built: the press held back for the question is handed
/// back to the shot select, whose own decide chooses the run, and the game builds it. What this asks is that
/// the run writes a replay of its own under the same name.
fn the_run_the_answer_let_through_writes_its_own_replay(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.image().a_run_has_been_chosen();
    game.builds_a_stage();
    game.frames_until("this run's own first chapter", 600, || {
        log.said_since(from, "stage 1 chapter 1 (stage start)")
    });
    game.frames_until_a_thread("this run's own replay", || {
        orb_core::replay::read_being_played(game.dir(), SLOT, &Th07)
            .is_some_and(|replay| replay.stages[0].chapters.len() == 1)
    });
}

/// はじめから with the setting off: the question goes up over the shot select, はい keeps the replay of the
/// run being written over, and the run starts afterwards.
#[test]
fn a_replay_of_th07_about_to_be_written_over_is_offered_and_keeping_it_lets_the_run_start() {
    in_its_own_process(|| {
        let game = asking("th07-replay-offered-keep");
        let log = game.log();
        a_run_left_behind(&game);
        started_over(&game);

        // What the question puts on the screen: what it asks, the two answers a line apart with the cursor
        // on はい — keeping is what a mistake here should not cost — and under them what the answer means.
        game.forget();
        game.frame();
        let asked = game.says("リプレイを保存する");
        let yes = game.says("はい");
        let no = game.says("いいえ");
        assert_eq!(
            (asked.len(), yes.len(), no.len()),
            (1, 1, 1),
            "the question is not the three lines it draws:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(yes[0].color, SELECTED, "the cursor is not on はい");
        assert_eq!(no[0].color, NORMAL);
        assert!(yes[0].y < no[0].y, "はい is not the first of the two");

        game.press_until(keys::Z, "はい answered", || {
            log.said("replay: kept on the keyboard")
        });
        let kept = format!("{SLOT}-{DATED}");
        assert_eq!(
            orb_core::replay::saved(game.dir()),
            vec![kept.clone()],
            "the replay the question kept is not among the saved:\n  {}",
            log.lines().join("\n  ")
        );
        // Whose it is travels with it, which is what the list shows against one.
        assert_eq!(
            orb_core::replay::read_saved(game.dir(), &kept, &Th07)
                .expect("the replay kept")
                .name,
            PLAYER,
        );

        // And the run the answer was given about starts and writes into a replay of its own: a question that
        // swallowed the press would be a launch that cannot start a run at all.
        the_run_the_answer_let_through_writes_its_own_replay(&game);
        assert_eq!(orb_core::replay::saved(game.dir()), vec![kept]);
    });
}

/// And いいえ: the file goes, and the run still starts.
#[test]
fn declining_takes_the_replay_of_th07_away_and_the_run_still_starts() {
    in_its_own_process(|| {
        let game = asking("th07-replay-offered-decline");
        let log = game.log();
        a_run_left_behind(&game);
        started_over(&game);

        // One press down from はい, after the frames the question reads nothing over.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "いいえ answered", || {
            log.said("replay: declined on the keyboard")
        });
        assert!(
            orb_core::replay::saved(game.dir()).is_empty(),
            "a replay that was declined was kept anyway",
        );
        assert!(
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th07).is_none(),
            "the file the run was being written into is still there",
        );
        // And this run writes a replay of its own under the same name, the one before it having gone.
        the_run_the_answer_let_through_writes_its_own_replay(&game);
    });
}

/// With the setting on, nothing is asked and the replay is kept: which is what the setting says, and the
/// reason it exists is a run that gets started over a dozen times in an evening.
#[test]
fn keeping_every_replay_of_th07_asks_nothing() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-replay-offered-never",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        a_run_left_behind(&game);

        let from = log.written();
        game.at_the_shot_select();
        game.press_until(keys::Z, "the question about the run left behind", || {
            log.said_since(from, "asking where to start")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said_since(from, "resume: from the beginning, answered on the keyboard")
        });

        // The replay is kept without anything having been asked, and the run starts.
        game.frames_until_a_thread("the replay kept", || {
            orb_core::replay::saved(game.dir()) == vec![format!("{SLOT}-{DATED}")]
        });
        assert!(
            !log.said("asking whether to keep it"),
            "a launch that keeps every replay asked about one:\n  {}",
            log.lines().join("\n  ")
        );
        the_run_the_answer_let_through_writes_its_own_replay(&game);
    });
}

/// **And a recipe answering it by item**, which is what a launch playing itself has to do: the cursor is
/// orb's own and nothing of it reaches a bot, so a press only ever takes `はい`.
///
/// Item 1 is `いいえ`, and what it does is take the file away. Which is the whole of why the step exists: the
/// answer a press cannot give is the one that loses something.
#[test]
fn a_recipe_declines_the_offer_over_th07_by_the_item_no_press_reaches() {
    in_its_own_process(|| {
        let game = asking("th07-replay-offered-to-a-recipe");
        let log = game.log();
        a_run_left_behind(&game);
        started_over(&game);

        // The question is up and the cursor is on はい, which is the answer a press would give. What the
        // recipe asks for is the item beside it.
        orb_core::runtime::answer_the_question(1);
        game.frames_until("いいえ answered on the recipe's own ask", 60, || {
            log.said("replay: declined on the recipe's own ask")
        });
        assert!(
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th07).is_none(),
            "the replay an answer of item 1 declined is still there:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            orb_core::replay::saved(game.dir()).is_empty(),
            "a replay that was declined was kept anyway",
        );
        // And the run the answer was on the way into still starts, whichever item answered it.
        the_run_the_answer_let_through_writes_its_own_replay(&game);
    });
}

/// **The screen a run ends at, which is where the game's own offer would have been.** A run that finished
/// reaches it, and the question orb asks there is the same one — on the frame the game's own offer is written
/// past, two screens after the name entry whoever finished the run typed at.
#[test]
fn a_finished_run_of_th07s_replay_is_offered_where_the_games_own_would_have_been() {
    in_its_own_process(|| {
        let game = asking("th07-replay-offered-at-the-end");
        let log = game.log();
        game.in_a_pointdevice_run();
        game.frames_until_a_thread("the chapter written down", || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });

        // The run over, the screen its own end arrives on, and that screen's walk reaching the question
        // about saving a replay — which is the one state of it orb writes.
        game.the_run_ends_at_its_own_screen();
        game.the_screen_offers_a_replay();
        game.frames_until("the game's own offer written past", 8, || {
            log.said("replay: the offer to save one was taken off the screen a run ended on")
        });
        assert!(
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            )),
            "the replay of the run that finished was not offered:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.a_frame_that_says(orb_core::replay_ui::asked(LANGUAGE))
                .len(),
            1,
            "the question is not on the screen the run ended at:\n  {}",
            log.lines().join("\n  ")
        );

        game.press_until(keys::Z, "はい answered", || {
            log.said("replay: kept on the keyboard")
        });
        assert_eq!(
            orb_core::replay::saved(game.dir()),
            vec![format!("{SLOT}-{DATED}")],
        );
        // And the file the run was written into is gone, having become the one that was kept: a run
        // finished with is not one anything adds to.
        assert!(
            !game.sim().files().holds(
                game.dir()
                    .join(KEPT_IN)
                    .join(format!("{SLOT}.{WHILE_PLAYING}"))
            ),
            "the run finished with is still being written into",
        );
    });
}
