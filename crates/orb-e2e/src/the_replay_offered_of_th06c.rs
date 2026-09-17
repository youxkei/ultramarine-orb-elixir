//! **Whether a run of th06c's replay is kept, which is the setting and the question behind it.**
//!
//! On, every one of them is kept and nothing is asked. Off is not *no replays*: it is orb asking, which is
//! where somebody says which of their runs were worth keeping — so the answer is a question orb puts up, and
//! いいえ is what takes the file away.
//!
//! **Asked where nothing more will ever be added to one**, which for this game is the screen a run ends at:
//! `Th06c::skip_replay_prompt` writes the game's own question about saving a replay away — a replay of the
//! game's own being the path that survived with nothing in it of the attempts that did not — and that same
//! frame is where orb's own goes up. The other moment is はじめから, where the run left unfinished is about
//! to be written over, and there it is kept without asking: the front end has taken its own job out and
//! there is no screen left to put a question over, which
//! `watching_a_pointdevice_replay_of_th06c` is the other side of.
//!
//! The question orb asks is its own screen, drawn through this game's Direct3D 11 — see
//! `the_mode_question_over_th06c` — and answered by the recipe the way every question of orb's is over a
//! game orb plays itself.

use orb_config::LogLevel;

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the two kinds.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name orb gives the run these play, which names its file.
const SLOT: &str = "normal-reimu-a";

/// How long each stage of these runs lasts, in the game's own frames, and how many frames a whole run of six
/// of them needs with room for the transitions, the ending and the entry after it.
const STAGE_FRAMES: u32 = 330;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * 6 * 3;

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 120_000;

/// A run played from the title to its clear, the entry named, and the question about the replay answered with
/// `item` where one is asked at all.
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
        // **The decide tapped until the title is up**, which is how the normal-mode clear's own recipe walks
        // these screens: three of them stand between the name and the title and each ignores a press for
        // frames of its own, so a count of presses is a launch that sits in one of them. Where orb asks
        // about the replay the question above is what the walk ends on instead.
        "- press: [decide]
  frames: 6
  until: { screen: 2 }
  give_up_after: 3600
"
        .to_owned()
    };
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ frames: {A_WHOLE_RUN} }}
- name: ORB
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
    game.stages_last(STAGE_FRAMES);
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return game;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
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

/// **With the setting on, nothing is asked and the replay is kept.**
#[test]
fn a_run_of_th06c_finished_with_the_setting_on_keeps_its_replay_unasked() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06c-offered-unasked", false, 0);
        let log = game.log();
        assert!(
            !log.said("is finished with; asking whether to keep it"),
            "a launch that keeps every replay asked about one:\n  {}",
            log.lines().join("\n  ")
        );
        let left = replays(&game);
        assert!(
            left.iter()
                .any(|path| path.contains(SLOT) && path.ends_with(ONCE_KEPT)),
            "the run finished left no replay anybody could watch: {left:?}",
        );
    });
}

/// **And with it off, the question goes up where the run's own screen would have asked.**
#[test]
fn a_run_of_th06c_finished_with_the_setting_off_is_asked_about() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06c-offered-asked", true, 0);
        let log = game.log();
        assert!(
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            )),
            "the run finished was not asked about:\n  {}",
            log.lines().join("\n  ")
        );
        // And the game's own question was written away on that same frame, which is what makes the frame
        // orb's to ask on — see `the_screen_a_finished_run_of_th06c_ends_at`.
        assert!(
            log.said("result: no replay of the game's own is offered for a run with chapters"),
            "the game's own question about a replay was left standing:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **はい keeps the file**, which is the answer the cursor starts on.
#[test]
fn a_replay_of_th06c_answered_for_is_kept() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06c-offered-kept", true, 0);
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
fn a_replay_of_th06c_declined_is_taken_away() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06c-offered-declined", true, 1);
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
