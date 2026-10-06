//! **Whether a run of th11's replay is kept, which is the setting and the question behind it.**
//!
//! On, every one of them is kept and nothing is asked. Off is not *no replays*: it is orb asking, which is
//! where somebody says which of their runs were worth keeping — so the answer is a question orb puts up, and
//! いいえ is what takes the file away.
//!
//! **Asked where nothing more will ever be added to one**, which for a clear of this game is the title's screen
//! 15 (0x43ffa0), the replay's slots after the clear's name entry: `Th11::skip_replay_prompt` takes the game's own
//! offer of a replay off that screen — a replay of the game's own being the path that survived with nothing in it
//! of the attempts that did not — and that same frame is where orb's own goes up. The other moment is はじめから,
//! where the run left unfinished is about to be written over, and there it is kept without asking — see
//! `the_replay_a_run_of_th11_leaves_behind`.
//!
//! **Played to the clear by the rules and named by the recipe**, the entry being one no count of presses types
//! a name into — see `Game::types_its_name` — so the name the kept replay carries is what the entry was left
//! holding.

use orb_config::LogLevel;
use orb_core::game::th11::Th11;

use crate::fake::th11::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the two kinds.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name orb gives the run these play — Normal, and Reimu with Yukari — which names its file.
const SLOT: &str = "normal-reimua";

/// **How long each stage of these runs lasts**, past the laid-out stage's midboss and its boss, beaten a little
/// before the stage ends — see `fake::th11`'s `THE_BOSS_FIGHTS_FOR` — and how many frames a whole run of six of
/// them is given, with room for the builds, the ending and the screens after it.
const STAGE_FRAMES: u32 = 4_500;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * 6 * 3;

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 120_000;

/// The name the recipe types into the game's own entry.
const NAMED: &str = "ORB";

/// **A run played from the title to its clear, the entry named, and the question about the replay answered
/// with `item`** where one is asked at all — the walks every case of this game's makes, then the whole run.
fn a_run_played_to_its_clear(name: &str, asking: bool, item: usize) -> Box<Fake> {
    let answer = if asking {
        format!(
            "- press: [decide]
  frames: 6
  until: {{ question: keep_replay }}
  give_up_after: 7200
- answer: {{ question: keep_replay, item: {item} }}
  give_up_after: 600
"
        )
    } else {
        // **The decide tapped until the main menu is up**, which is how a clear of this game is walked out:
        // the entry's end, and the slots after it, which orb takes off as they come up.
        "- press: [decide]
  frames: 6
  until: { screen: 2 }
  give_up_after: 7200
"
        .to_owned()
    };
    let recipe = format!(
        "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ frames: {A_WHOLE_RUN} }}
- name: {NAMED}
  give_up_after: 3600
{answer}\
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = !asking;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(STAGE_FRAMES);
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return game;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the recipe never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  ")
    );
}

/// Every file orb left under its own replay directory.
fn replays(game: &Fake) -> Vec<String> {
    game.sim()
        .files()
        .paths()
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| path.contains(KEPT_IN))
        .collect()
}

/// **With the setting on, nothing is asked and the replay is kept** — under the name the entry was left
/// holding.
#[test]
fn a_run_of_th11_finished_with_the_setting_on_keeps_its_replay_unasked() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th11-offered-unasked", false, 0);
        let log = game.log();
        assert!(
            !log.said("is finished with; asking whether to keep it"),
            "a launch that keeps every replay asked about one:\n  {}",
            log.lines().join("\n  ")
        );
        let kept = orb_core::replay::saved(game.dir());
        let [kept] = kept.as_slice() else {
            panic!(
                "the run finished left no replay anybody could watch: {:?}",
                replays(&game)
            );
        };
        assert!(
            kept.starts_with(SLOT),
            "the replay kept is not the run's own: {kept}"
        );
        let replay =
            orb_core::replay::read_saved(game.dir(), kept, &Th11).expect("the replay kept");
        assert_eq!(
            replay.name, NAMED,
            "the replay kept does not carry the name the game's own entry was left holding",
        );
    });
}

/// **And with it off, the question goes up where the game's own offer would have.**
#[test]
fn a_run_of_th11_finished_with_the_setting_off_is_asked_about() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th11-offered-asked", true, 0);
        let log = game.log();
        assert!(
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            )),
            "the run finished was not asked about:\n  {}",
            log.lines().join("\n  ")
        );
        // And the game's own offer was taken off on that same frame, which is what makes the frame orb's to
        // ask on.
        assert!(
            log.said("replay: the offer to save one was taken off the screen a run ended on"),
            "the game's own offer of a replay was left standing:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **はい keeps the file**, which is the answer the cursor starts on.
#[test]
fn a_replay_of_th11_answered_for_is_kept() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th11-offered-kept", true, 0);
        let log = game.log();
        assert!(
            log.said("replay: kept on the"),
            "はい was answered and nothing was kept:\n  {}",
            log.lines().join("\n  ")
        );
        let left = replays(&game);
        assert!(
            left.iter()
                .any(|path| path.contains(SLOT) && path.ends_with(ONCE_KEPT)),
            "the replay answered for is not a file anybody could watch: {left:?}",
        );
    });
}

/// **And いいえ takes it away**, which is the whole point of being asked.
#[test]
fn a_replay_of_th11_declined_is_taken_away() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th11-offered-declined", true, 1);
        let log = game.log();
        assert!(
            log.said("replay: declined on the"),
            "いいえ was answered and nothing said so:\n  {}",
            log.lines().join("\n  ")
        );
        let left = replays(&game);
        assert!(
            !left.iter().any(|path| path.ends_with(ONCE_KEPT)),
            "a replay nobody wanted was kept all the same: {left:?}",
        );
        assert!(
            !left.iter().any(|path| path.ends_with(WHILE_PLAYING)),
            "a replay nobody wanted was left half written: {left:?}",
        );
    });
}
