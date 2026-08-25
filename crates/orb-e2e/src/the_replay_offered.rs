//! **Whether a run's replay is kept, which is the setting and the question behind it.**
//!
//! On, every one of them is kept and nothing is asked. Off is not *no replays*: it is orb asking, which is
//! where somebody says which of their runs were worth keeping — so the answer is a question orb puts up, and
//! いいえ is what takes the file away.
//!
//! **Asked at the two moments nothing more will ever be added to one.** The screen a run ends at, which is
//! where the game's own offer would have been — written past, a replay of the game's own being the path that
//! survived with nothing in it of the attempts that did not — and はじめから, where the run left unfinished
//! is about to be written over. Both are the last moment there is: after either, the file would be a replay
//! of a run nobody can add to and nobody asked to keep.
//!
//! The question orb asks is its own screen, and what it reads its keys with is what every question of orb's
//! reads — so a pad answers it, which is `mode_on_the_pad.rs`'s subject rather than this one's.

use crate::fake::th06::{CARD_STARTS, Fake, STAGES, the_run};
use crate::fake::{LANGUAGE, Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{Scene, Screen};
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

/// Where orb keeps them, and what it calls the run being played.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";

const SLOT: &str = "normal-reimu-a";
/// The date the simulated host's calendar is set to, which a kept replay's name carries.
const DATED: &str = "20040815-214233";

/// How long each stage of a run that is cleared lasts, and the frames a whole run of six needs — the shape
/// `the_screen_a_finished_run_ends_at.rs` uses, a clear being the only way a run of this mode reaches the
/// screen it would be asked about at.
const STAGE_FRAMES: u32 = 300;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// A launch that asks, which is what a file that says nothing about it gives: `save_replays` off.
fn asking(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        config.save_replays = false;
        config.player_name = "ゆうか".to_owned();
    })
}

/// A run played as far as its midboss's card and then given up at the retry menu, which leaves both files
/// behind — the chapter to be picked up, and the replay to be asked about.
fn a_run_left_behind(game: &Fake) {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
    });
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
    game.frames_until_a_thread("the replay written down", || {
        orb_core::replay::read_being_played(game.dir(), SLOT, &Th06).is_some()
    });
}

/// The same run started again and はじめから answered, which is where the question about the replay of the
/// run being written over goes up.
fn started_over(game: &Fake) {
    let log = game.log();
    game.frames_until("the title menu ready to act on a press", 300, || {
        let front = game.image().front_end_now();
        front.screen == Screen::Title && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered again", || {
        game.image().front_end_now().screen == Screen::ShotType
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        game.image().front_end_now().acts_on_a_press()
    });
    game.press(keys::Z);
    game.frames(READS_KEYS_AFTER);
    game.press(keys::DOWN);
    game.press_until(keys::Z, "はじめから answered", || {
        log.said("resume: from the beginning, answered on the keyboard")
    });
    game.frames_until("the question about the replay", 60, || {
        log.said(&format!(
            "replay: {SLOT} is being written over; asking whether to keep it"
        ))
    });
}

/// はじめから with the setting off: the question goes up over the shot type select, はい keeps the replay of
/// the run being written over, and the press that starts the new run is handed on afterwards.
#[test]
fn a_replay_about_to_be_written_over_is_offered_and_keeping_it_starts_the_run() {
    in_its_own_process(|| {
        let game = asking("replay-offered-keep");
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
            "the question is not the three lines it draws",
        );
        assert_eq!(yes[0].color, SELECTED, "the cursor is not on はい");
        assert_eq!(no[0].color, NORMAL);
        assert!(yes[0].y < no[0].y, "はい is not the first of the two");
        // The game underneath is frozen: the run has not started, the press that would start it being held
        // back for this.
        assert!(!game.state().playing, "the run started under the question");

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
            orb_core::replay::read_saved(game.dir(), &kept, &Th06)
                .expect("the replay kept")
                .name,
            "ゆうか",
        );

        // And the press held back for the question is handed on, so the run the answer was given about
        // starts: a question that swallowed it would be a launch that cannot start a run at all.
        game.frames_until(
            "the run the question was answered on the way into",
            90,
            || game.state().playing,
        );
    });
}

/// And いいえ: the file goes, and the run still starts.
#[test]
fn declining_takes_the_replay_away_and_the_run_still_starts() {
    in_its_own_process(|| {
        let game = asking("replay-offered-decline");
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
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th06).is_none(),
            "the file the run was being written into is still there",
        );
        game.frames_until(
            "the run the question was answered on the way into",
            90,
            || game.state().playing,
        );
        // And this run writes a replay of its own under the same name, the one before it having gone.
        game.frames_until("this run's own first chapter", 400, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        game.frames_until_a_thread("this run's own replay", || {
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th06).is_some()
        });
    });
}

/// With the setting on, nothing is asked and the replay is kept: which is what the setting says, and the
/// reason it exists is a run that gets started over a dozen times in an evening.
#[test]
fn keeping_every_replay_asks_nothing() {
    in_its_own_process(|| {
        let game = Fake::attach("replay-offered-never", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        });
        let log = game.log();
        a_run_left_behind(&game);

        game.frames_until("the title menu ready to act on a press", 300, || {
            let front = game.image().front_end_now();
            front.screen == Screen::Title && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered again", || {
            game.image().front_end_now().screen == Screen::ShotType
        });
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said("resume: from the beginning, answered on the keyboard")
        });

        // The run starts without anything having been asked, and the replay is kept all the same.
        game.frames_until("the run started", 90, || game.state().playing);
        assert!(
            !log.said("asking whether to keep it"),
            "a launch that keeps every replay asked about one:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            orb_core::replay::saved(game.dir()),
            vec![format!("{SLOT}-{DATED}")],
        );
    });
}

/// **And a recipe answering it by item**, which is what a launch playing itself has to do: the cursor is
/// orb's own and nothing of it reaches a bot, so a press only ever takes `はい` — three launches on the real
/// game were written to press `down` first and every one of them answered with the item the cursor was on.
///
/// Item 1 is `いいえ`, and what it does is take the file away. Which is the whole of why the step exists: the
/// answer a press cannot give is the one that loses something.
#[test]
fn a_recipe_declines_the_offer_by_the_item_no_press_reaches() {
    in_its_own_process(|| {
        let game = asking("replay-offered-to-a-recipe");
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
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th06).is_none(),
            "the replay an answer of item 1 declined is still there:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            orb_core::replay::saved(game.dir()).is_empty(),
            "a replay that was declined was kept anyway",
        );
        // And the run the answer was on the way into still starts, the press held back for the question
        // being handed on whichever item answered it.
        game.frames_until(
            "the run the question was answered on the way into",
            90,
            || game.state().playing,
        );
    });
}

/// **The screen a run ends at, which is where the game's own offer would have been.** A run that clears
/// reaches it, and the question orb asks there is the same one — after the name entry and the stats screen,
/// which are the screens whoever just finished the run reads.
#[test]
fn a_finished_runs_replay_is_offered_where_the_games_own_would_have_been() {
    in_its_own_process(|| {
        // A run cleared, which is the one way a run of this mode reaches that screen — and with what it
        // pressed still written down, unlike `--clear`, which refuses to write anything about a run whose
        // player could not be hit. What is being driven here is the screen and not the clear.
        let game = Fake::attach("replay-offered-at-the-end", the_run(), |config| {
            config.fast_clear = true;
            config.log_level = LogLevel::Verbose;
            config.save_replays = false;
        });
        let log = game.log();
        game.stages_last(STAGE_FRAMES);
        // Nothing asked at the title menu: a launch that clears a run takes the mode it is given, so the
        // press there goes straight to the game's own screens — see *Pointdevice and normal*.
        game.in_a_run_nobody_was_asked_about();
        game.puts_a_bullet_on_the_player();

        game.frames_until("the result screen", A_WHOLE_RUN, || {
            game.image().scene() == Scene::Result
        });
        game.frames_until("the game's own offer written past", A_WHOLE_RUN, || {
            log.said("result: no replay of the game's own is offered for a run with chapters")
        });
        assert!(
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            )),
            "the replay of the run that finished was not offered:\n  {}",
            log.lines().join("\n  ")
        );
        game.forget();
        game.frame();
        assert_eq!(
            game.says(orb_core::replay_ui::asked(LANGUAGE)).len(),
            1,
            "the question is not on the screen the run ended at",
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

/// The name whoever finished the run left in the game's own entry, and the one the file was carrying before
/// it: two names, because a file that already agreed with the entry could not say which of the two it took.
const TYPED: &str = "YXK";
const CARRIED: &str = "ゆうか";

/// **And a kept replay is named by the game's own name entry**, which is the screen this question comes up
/// two screens behind: what somebody typed there for the score is the name the game's own replay of that run
/// would have been saved under, so it is the name orb's own carries too.
#[test]
fn a_kept_replay_carries_the_name_the_games_own_entry_holds() {
    in_its_own_process(|| {
        let game = Fake::attach("replay-named-by-the-game", the_run(), |config| {
            config.fast_clear = true;
            config.log_level = LogLevel::Verbose;
            config.save_replays = false;
            config.player_name = CARRIED.to_owned();
        });
        let log = game.log();
        game.stages_last(STAGE_FRAMES);
        game.in_a_run_nobody_was_asked_about();
        game.puts_a_bullet_on_the_player();

        game.frames_until("the result screen", A_WHOLE_RUN, || {
            game.image().scene() == Scene::Result
        });
        // The entry as whoever finished the run left it: the characters they chose and spaces to the end of
        // the field. The typing itself is the game's own — see `Image::the_name_its_entry_holds`.
        game.image().the_name_its_entry_holds(TYPED);
        game.frames_until("the game's own offer written past", A_WHOLE_RUN, || {
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            ))
        });
        game.forget();
        game.press_until(keys::Z, "はい answered", || {
            log.said("replay: kept on the keyboard")
        });

        let kept = format!("{SLOT}-{DATED}");
        assert_eq!(
            orb_core::replay::read_saved(game.dir(), &kept, &Th06)
                .expect("the replay kept")
                .name,
            TYPED,
            "the replay is not named by the game's own entry:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
