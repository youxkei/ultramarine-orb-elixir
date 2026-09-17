//! **The card records of th06c across a run picked up: one attempt counted for the landing, and the
//! captures the run had put back with it.**
//!
//! This game keeps sixty-four card records of its own, and its stage callback fills them from the generator
//! 2048 draws deep before the file is parsed over them — `0x140024230` to `0x1400247e5`. Two things of orb's
//! touch them where a run is picked up:
//!
//! - **`Th06c::count_card_attempt`** adds one to the record of the card that is up, because picking a run up
//!   where it was left is an attempt at that chapter — the same as the retry menu's, and the game counts
//!   neither: it counts where a card *starts*, and a landing is inside one. It counts nothing where no card
//!   is up, `CURRENT_CARD` holding the last card a boss was on with nothing clearing it, and nothing against
//!   a record the game has not named, the bytes of an unnamed one being the generator's.
//! - **`Th06c::captures` and `set_captures`** put the history back as it was before the buttons went in:
//!   playing a stage again starts every card the run had passed and the game counts an attempt at each of
//!   them, so what a playback counted is not what the run did. The names those starts wrote stay, a name
//!   being what the playback learned rather than what it counted.
//!
//! **And what writing these found was a name the game could not have written.** The laid-out game's card
//! start filled the whole `0x24`-byte name field with the card's own name — twelve characters of UTF-8,
//! exactly that many bytes — so the span held no terminator, `name_in` answered nothing and every landing
//! inside a card was refused. A copy into a fixed field always leaves one, which is what the read looks for:
//! a span with no nought in it is the generator's noise and not a name. The fake cuts the name to the field
//! now and sums the bytes that went in.
//!
//! **Which is a pick-up and not a retry.** Both of those are on the playback's own path — `resume::landed`
//! is what the frame after one reads — and a retry puts a chapter back out of a snapshot of the game's whole
//! memory, where the card records come back with everything else and no attempt is counted for the landing
//! because there is none. So every launch here is two: one that leaves a chapter, and one that finds it.
//!
//! **And what is not here is the other refusal**, `score: card N is not one the game has named`. A card that
//! is up in a run of this game is one the game's own start named, that start being where a name is written at
//! all — so a landing inside a card whose record is still the generator's is not a run this game plays, and
//! the only way to ask for one is a fake that loses the name the playback wrote. Which is the defect
//! `set_captures_keeping_names` was written for: what a test over that path would hold is the fake against
//! itself, and the refusal's own reading is th06 1.02h's, whose records these are. The test below asks
//! instead that the refusal never fires, which is the claim about this game that means anything.

use std::path::{Path, PathBuf};

use orb_config::LogLevel;

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file: this game's own first difficulty, Reimu, and
/// her first shot — nothing here walks the difficulty select, so the run is the one a front end left alone
/// starts.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// The chapter a run is left in for each of these: the card the midboss puts up, and the stage's own start,
/// which is outside every card.
const INSIDE_A_CARD: u32 = 3;
const OUTSIDE_EVERY_CARD: u32 = 1;

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

/// **A run left in the chapter named**, and the files that launch left behind.
fn a_run_left_in_chapter(name: &str, chapter: u32) -> Vec<(PathBuf, Vec<u8>)> {
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: {chapter} }}
  give_up_after: 18000
- leave: true
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
            config.save_replays = true;
        },
    );
    played_out(&game);
    let files = left_by(&game, game.launch().dir());
    assert!(
        files.iter().any(|(inside, _)| {
            inside.to_string_lossy().contains(SLOT)
                && inside.to_string_lossy().contains("pointdevice_resume")
        }),
        "no chapter file of {SLOT} was left: {:?}",
        files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
    );
    files
}

/// And that run picked up again, played out, with everything its launch said.
fn picked_up(name: &str, files: &[(PathBuf, Vec<u8>)]) -> Box<Fake> {
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- wait_until: {{ frames: 600 }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = Fake::attach_finding(
        Display::agreed(120),
        name,
        files,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    played_out(&game);
    assert!(
        game.log().said("resume: from where it stopped"),
        "the run was not picked up where it stopped:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// **One attempt at the card the chapter is on**, counted for the landing and said.
///
/// **Which the playback's own card start is what makes possible.** The rebuild of a picked-up stage fills
/// all sixty-four records from the generator — the callback's 2048 draws — so nothing in the array is a name
/// until a card starts in that stage; the buttons fed then reach the frame the card started on and the
/// game's own start at `0x1400149c9` writes its name in. `set_captures_keeping_names` is what leaves that
/// name standing while it puts the counts back, and without it the landing would be asked about a record
/// whose bytes are noise.
#[test]
fn a_run_of_th06c_picked_up_inside_a_card_counts_one_attempt_at_it() {
    in_its_own_process(|| {
        let left = a_run_left_in_chapter("th06c-the-cards-left-in-one", INSIDE_A_CARD);
        let game = picked_up("th06c-the-cards-an-attempt", &left);
        assert!(
            game.log().said("resume: attempt "),
            "the landing counted no attempt at the card its chapter is on:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            !game.log().said("is not one the game has named"),
            "the card the landing is inside was read as one the game had not named:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And the captures the run had are put back with it, the playback counting none of its own.
#[test]
fn a_run_of_th06c_picked_up_carries_the_captures_it_had() {
    in_its_own_process(|| {
        let left = a_run_left_in_chapter("th06c-the-cards-left-with-captures", INSIDE_A_CARD);
        let game = picked_up("th06c-the-cards-the-captures", &left);
        assert!(
            game.log().said("byte(s) of captures put back"),
            "the run came back with no captures of its own:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And nothing is counted where no card is up, which is what `CURRENT_CARD` holding the last one needs.
#[test]
fn a_run_of_th06c_picked_up_outside_every_card_counts_no_attempt() {
    in_its_own_process(|| {
        let left = a_run_left_in_chapter("th06c-the-cards-left-outside", OUTSIDE_EVERY_CARD);
        let game = picked_up("th06c-the-cards-no-attempt", &left);
        assert!(
            game.log()
                .said("score: no spell card is up; no attempt counted"),
            "a landing outside every card was not read as one:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            !game.log().said("resume: attempt "),
            "a landing outside every card counted an attempt all the same:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
