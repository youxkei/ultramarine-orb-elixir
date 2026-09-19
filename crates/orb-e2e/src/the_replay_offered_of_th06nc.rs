//! **Whether a run of th06nc's replay is kept, which the screen a run ends at is where orb asks.**
//!
//! The same four claims `the_replay_offered_of_th06c` makes of that game: with the setting on every replay
//! is kept unasked; with it off orb asks, on the frame the game's own offer is written away; はい makes the
//! file final and いいえ takes it away.
//!
//! **What makes the frame orb's is `Th06nc::skip_replay_prompt`**, which writes the screen's way out —
//! state 17 — over the state its own offer stands in: state 15, the one the name entry hands on to. A
//! launch that held every state of a run that had ended photographed
//! **「今回のリプレイを保存しますか？」** there. A pointdevice run's replay is the buttons without the
//! rewinds between them, so the game's own offer is one nothing could accept: a file kept from it plays
//! back as a run dying where this one carried on.
//!
//! **These tests replace the record of a decline.** Until state 15 was read the method answered false,
//! `runtime`'s one call to `the_replay_of_a_run_finished_with` sat behind it, and a cleared run of this
//! game left a `.pending` nobody could watch — the two moments a replay is made final being this screen
//! and the answer that writes over the chapter a run was left in, of which this game had only the second.
//!
//! **And the write is not driven on the real game**: `runtime` asks the method in Pointdevice mode alone,
//! a pointdevice run reaches that screen by a clear only — every death of one is answered by the retry and
//! a run given up at it walks the front end's own way back — and `--clear` over this game stands still in
//! stage 1's boss fight. So the laid-out game is what holds the claim; see the change's description and
//! `th06nc/asm.md`'s *The entry a name is typed into*.

use orb_config::LogLevel;

use crate::fake::th06nc::{Fake, TO_A_RUN};
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
/// of them needs with room for the transitions, the ending and the screen after it.
const STAGE_FRAMES: u32 = 330;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * 6 * 3;

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 120_000;

/// A run played from the title to its clear, the entry named, and the question about the replay answered with
/// `item` where one is asked at all.
///
/// **Played rather than cleared on demand**, where `a_clear_on_demand_of_th06nc` uses `--clear`: that flag
/// turns the *keeping* off — `resume` false and `block_replay_save` set — so a launch under it records no
/// replay and leaves nothing for a question to be about. **The chapters it does not turn off**, which a
/// launch of `--clear` over the real game says: `stage 1 chapter 4`, `5` and `6` went down inside its
/// boss's own fight.
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
        // **The decide tapped until the title is up**, which is how this game's own whole-run recipes walk
        // these screens: the offer and the screens after it each ignore a press for frames of their own, so
        // a count of presses is a launch that sits in one of them.
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

/// **With the setting on, nothing is asked and the replay is kept** — which is also the claim that a run of
/// this game is finished with at all, where before state 15 was read it never was.
#[test]
fn a_run_of_th06nc_finished_with_the_setting_on_keeps_its_replay_unasked() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06nc-offered-unasked", false, 0);
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

/// **And with it off, the question goes up where the game's own offer would have stood.**
#[test]
fn a_run_of_th06nc_finished_with_the_setting_off_is_asked_about() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06nc-offered-asked", true, 0);
        let log = game.log();
        assert!(
            log.said(&format!(
                "replay: {SLOT} is finished with; asking whether to keep it"
            )),
            "the run finished was not asked about:\n  {}",
            log.lines().join("\n  ")
        );
        // And the game's own offer was written away on that same frame, which is what makes the frame
        // orb's to ask on — see `Th06nc::skip_replay_prompt`.
        assert!(
            log.said("result: no replay of the game's own is offered for a run with chapters"),
            "the game's own offer to save a replay was left standing:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **はい keeps the file**, which is the answer the cursor starts on.
#[test]
fn a_replay_of_th06nc_answered_for_is_kept() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06nc-offered-kept", true, 0);
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

/// **And いいえ takes it away**, which is the whole point of being asked — and what a `.pending` left behind
/// would otherwise be.
#[test]
fn a_replay_of_th06nc_declined_is_taken_away() {
    in_its_own_process(|| {
        let game = a_run_played_to_its_clear("th06nc-offered-declined", true, 1);
        let log = game.log();
        assert!(
            log.said("replay: declined on the"),
            "いいえ was answered and nothing said so:\n  {}",
            log.lines().join("\n  ")
        );
        let left = replays(&game);
        assert!(
            !left
                .iter()
                .any(|path| path.ends_with(ONCE_KEPT) || path.ends_with(WHILE_PLAYING)),
            "the replay declined was left behind: {left:?}",
        );
    });
}
