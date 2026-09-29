//! **What a chapter of th06nc retried leaves in the record of a run, and whether a playback of that record is
//! the run.**
//!
//! A retry is not a thing the record knows about: the buttons are one entry per stage frame, and an attempt
//! that is rewound plays the same frames again and writes over them — so what the file holds is the frames of
//! the attempt that survived, stitched onto the frames before the chapter it was retried at. **But the
//! surviving attempt was played from a snapshot put back at the chapter's own frame**, where a playback of the
//! file reaches that frame by playing the buttons under it instead: whatever the restore leaves standing from
//! the attempt that was lost is state the surviving attempt was played with and a playback has never seen.
//!
//! `the_run_a_retried_chapter_of_th06c_writes_down.rs` is the same question of that game. What this one adds
//! is a longer list of what a restore leaves standing on purpose — the renderer's objects, the sound
//! library's block, and the heap regions `a_span_of_th06nc_held_back_by_its_source.rs` names — and every one
//! of them is state of exactly that kind.
//!
//! **Two launches, each played by a recipe**, as every run of this game that is picked up is. **What is not
//! the recipe's is the death**: the hit is this test's, on the frame it chooses, with the stage firing nothing
//! and the rules forbidden the bomb — so the chapter retried is the card's and no other.

use std::path::{Path, PathBuf};

use crate::fake::th06nc::{ATTACK_CHANGES, CARD_STARTS, Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::State;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and a wait inside one.
const FRAMES: u32 = 30_000;

/// The frames played into the card's chapter before the hit, so that what the retry writes over is frames of
/// buttons and not the chapter's own first one.
const INTO_THE_CHAPTER: u32 = 60;

/// A launch playing `recipe`, finding `files` under its directory as a launch before left them, with nothing
/// fired at the player — `self_check` on, and every replay kept without asking, as th06c's is.
fn launched(name: &str, files: &[(PathBuf, Vec<u8>)], recipe: String) -> Box<Fake> {
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.self_check = true;
            config.save_replays = true;
        },
    );
    game.fires_at_the_player(false);
    game
}

/// The recipe run out, the game asked to close by its last step.
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

/// A chapter retried, and the run picked up out of the file the chapter after it left.
#[test]
fn a_run_of_th06nc_whose_chapter_was_retried_is_put_back_at_the_chapter_after_it() {
    in_its_own_process(|| {
        // The run into its fourth chapter — the attack after the card — and left there, with the bomb
        // forbidden before the stage: the rules spend one on the frame the player is hit.
        let played = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- bombs: false
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ stage: 1, chapter: 4 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let first = launched("th06nc-retried", &[], played);
        let log = first.log();

        // The chapter that is going to be retried, with frames of its own behind it, and the state it began
        // with read out of the game's memory: that is what a restore has to put back, field for field.
        first.frames_until("the card's chapter", FRAMES, || {
            log.lines().iter().any(|line| {
                line.contains("chapter 3 at frame") && line.contains("a midboss spellcard")
            })
        });
        let at_the_card = first.state();
        assert!(
            at_the_card.stage_frames >= CARD_STARTS,
            "the card's chapter began before the script put the card up",
        );
        first.frames(INTO_THE_CHAPTER);

        // The player lost, and the chapter taken again: the memory is put back, so the run is where the
        // chapter began — but for the deaths, which orb counts itself and a restore leaves where the hit put
        // them.
        first.hit();
        first.frames_until("the death seen", 16, || log.said("died in chapter 3"));
        first.frames_until("the chapter put back", FRAMES, || {
            log.said("retry chapter 3 (retry 1)")
        });
        assert_eq!(
            State {
                deaths: at_the_card.deaths,
                ..first.state()
            },
            at_the_card,
            "the run is not where the chapter began, field for field",
        );

        // And the chapter after it, which is the file this run leaves.
        first.frames_until("the chapter after the retry", FRAMES, || {
            log.said("chapter 4 at frame")
        });
        let at = first.state().stage_frames;
        assert!(
            at >= ATTACK_CHANGES,
            "the chapter after the card began before the script moved the fight on",
        );
        first.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });
        played_out(&first);
        let files = left_by(&first, first.launch().dir());
        drop(first);

        // Picked up, which is where a stitched record tells.
        let picked_up = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- wait_until: {{ frames: 60 }}
- quit: true
"
        );
        let second = launched("th06nc-retried-then-picked-up", &files, picked_up);
        let log = second.log();
        second.frames_until("the run landed", FRAMES, || {
            log.said("resume: the landing is")
        });
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            second.state().stage_frames,
            at,
            "the run came back at another frame of the stage:\n  {}",
            log.lines().join("\n  ")
        );
        played_out(&second);
    });
}
