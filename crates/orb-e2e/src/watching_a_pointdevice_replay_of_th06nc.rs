//! **An attempt out of a Pointdevice replay of th06nc watched, off orb's own lists.**
//!
//! This game can record no replay of its own that a Pointdevice run would fit — its recorder is a thing a
//! chapter's restore rewinds — so the main menu's replays item puts **orb's** list up in place of the
//! game's own screen, and what that list offers is four levels deep: the replay, the stage inside it, the
//! chapter inside that, and which attempt at that chapter.
//!
//! **The item is the menu's fourth and the screen behind it is state 16**: item 4's case at `0x140049d8d`
//! fills the screen's table and writes `0xf` into the front end's state, state 15's case writes 16, and 16
//! is the list — `0x14004a042` formats `replay/th6_%.2d.rpy` sixty times into it. orb's question stands
//! over the *item*, the press that would open that screen being held back, so for a pointdevice run the
//! game never reaches its own list at all. `the_mode_question_over_th06nc` is what the drawing of a list
//! of orb's is held to; `watching_a_pointdevice_replay_of_th06c` is the same subject over the other 64-bit
//! build, whose item is its title's.
//!
//! **Three launches, because a replay has to be made final before anybody can watch it.** A run being
//! played writes a `.pending`, and what turns that into a file the lists offer is either a run reaching
//! the screen it ends at or the next run being started over it — see
//! `the_replay_a_run_of_th06nc_leaves_behind`. So the first launch leaves a run, the second answers
//! はじめから over it, which keeps the one before without asking, and the third is the one that watches.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;
use orb_core::game::th06nc::image::MENU_ITEM_REPLAYS;

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names both its files.
const SLOT: &str = "normal-reimu-a";

/// Where orb keeps the replays, and the extension of one that has been made final.
const KEPT_IN: &str = "pointdevice_replay";
const ONCE_KEPT: &str = "msgpack";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// A launch played out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// What a launch left under orb's directory, re-rooted for the next.
fn left_by(game: &Fake, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            let inside = path.strip_prefix(dir).ok()?.to_path_buf();
            Some((inside, files.get(&path)?))
        })
        .collect()
}

/// A launch with `files` under its directory and `recipe` playing it.
fn launched(name: &str, files: &[(PathBuf, Vec<u8>)], recipe: String) -> Box<Fake> {
    Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
}

/// **A kept replay of a run of this game**, and the files the launch that made it final left behind.
fn a_kept_replay() -> Vec<(PathBuf, Vec<u8>)> {
    let left = format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let first = launched("th06nc-watched-left", &[], left);
    played_out(&first);
    let files = left_by(&first, first.launch().dir());
    drop(first);

    // はじめから, which is the run the game was going to start: the replay of the run being written over is
    // kept as it goes, without asking.
    let over_it = format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 1 }}
  give_up_after: 600
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- wait_until: {{ frames: 120 }}
- quit: true
"
    );
    let second = launched("th06nc-watched-kept", &files, over_it);
    played_out(&second);
    assert!(
        second
            .log()
            .said(&format!("replay: kept as {KEPT_IN}/{SLOT}-")),
        "the run started over kept no replay of the one before it:\n  {}",
        second.log().lines().join("\n  ")
    );
    let files = left_by(&second, second.launch().dir());
    assert!(
        files.iter().any(|(inside, _)| {
            let said = inside.to_string_lossy();
            said.contains(KEPT_IN) && said.contains(SLOT) && said.ends_with(ONCE_KEPT)
        }),
        "no replay anybody could watch was left: {:?}",
        files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
    );
    files
}

/// **The four lists walked and the attempt watched**, which is the whole of what the replays item does for
/// a game orb records the replays of.
#[test]
fn an_attempt_of_a_kept_replay_of_th06nc_is_watched_off_orbs_own_lists() {
    in_its_own_process(|| {
        let files = a_kept_replay();
        let item = MENU_ITEM_REPLAYS;
        let watching = format!(
            "- wait_until: {{ screen: 2 }}
  give_up_after: 36000
- wait_until: {{ frames: 60 }}
- press: [decide]
  frames: 6
- wait_until: {{ screen: 3 }}
  give_up_after: 1800
- press: [down]
  frames: 6
  until: {{ screen: 3, cursor: {item} }}
  give_up_after: 900
- wait_until: {{ frames: 30 }}
- press: [decide]
  frames: 6
  until: {{ question: mode }}
  give_up_after: 1800
- answer: {{ question: mode, item: 0 }}
  give_up_after: 300
- answer: {{ question: choose_replay, item: 0 }}
  give_up_after: 900
- answer: {{ question: choose_replay, item: 0 }}
  give_up_after: 900
- answer: {{ question: choose_replay, item: 0 }}
  give_up_after: 900
- answer: {{ question: choose_replay, item: 0 }}
  give_up_after: 900
- wait_until: {{ question: choose_replay }}
  give_up_after: 3600
- quit: true
"
        );
        let game = launched("th06nc-watched", &files, watching);
        played_out(&game);
        let log = game.log();

        // The list in the game's screen's place, and then each level of it: the replay, the stage inside
        // it, the chapter inside that, and the attempts at that chapter.
        for said in [
            "replay: the replays to watch, over the screen the game keeps its own on",
            "replay: normal-reimu-a",
            "stage(s) in it",
            "chapter(s) in it",
            "attempt(s) at it",
        ] {
            assert!(
                log.said(said),
                "nothing said {said:?}:\n  {}",
                log.lines().join("\n  ")
            );
        }
        // And the attempt watched, which is a stage of the game built and the buttons of that attempt fed
        // into it — the run it is watched in being nobody's.
        assert!(
            log.said("replay: watching attempt "),
            "no attempt was watched:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("replay: the front end is asked for a run of stage 1"),
            "the attempt was never asked for a stage to be played back into:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
