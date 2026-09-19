//! **Whether a th06nc laid out by hand answers the same world twice from the same seed and the same
//! buttons.**
//!
//! Which is the question under every claim about putting a run of this game back: a playback feeds the
//! file's buttons to the game's own updates and expects the world the run had, and that expectation is
//! only meaningful if the game *has* one world for a given seed and a given sequence of buttons. A fake
//! that answered differently twice would make reproduction untestable here whatever orb did, and the real
//! game would be the only authority for it.
//!
//! **th06c's own module says why this exists**, a playback of that game having come out of step with
//! neither cause ruled out; what is this game's own is that its pick-up went wrong twice in the same
//! session and both were orb's — `Th06nc::run_chosen` reading a clock the frame it is asked at has
//! already stepped, and `Th06nc::start_stage` writing a stage nought-counted into a field this game
//! counts from one. Neither would have been told from a fake that answers a different world each launch,
//! which is what this asks about.
//!
//! **No rules in the recipe**, which is what makes the two runs comparable at all: `take_hits` and the
//! steering are a chance apiece, so a recipe carrying them is two different runs by design. What is left
//! is the walk and `play_until`, both of which are the same frames every time.
//!
//! **And `play_until` counts frames rather than chapters**, where th06c's counts to its third: what the
//! two launches are compared on is a frame apiece, and a chapter is a boundary the run's own play decides
//! — this game's `chapters.rs` has seven rows with nothing in them, so what divides a stage of it is the
//! card the laid-out stage casts and nothing put there by hand. A count of frames is the same count in
//! both launches whatever those boundaries come out as.

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How far into the stage both launches play, which is long enough for the bullets and the enemies of it
/// to have been through several of their own states.
const INTO_THE_STAGE: u32 = 600;

/// The recipe both launches play: the walk to a run, the run played into the stage, and out.
fn twice() -> String {
    format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ frames: {INTO_THE_STAGE} }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    )
}

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(twice());
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
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

/// The run's own world, frame by frame, as the runtime's status line carries it — the timestamp cut off,
/// that being the one part of the line no two launches share.
fn the_world_each_frame(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains(" scene=") && line.contains(" seed="))
        .map(|line| match line.split_once("] ") {
            Some((_, rest)) => rest.to_owned(),
            None => line,
        })
        .collect()
}

#[test]
fn a_run_of_th06nc_played_twice_from_the_same_seed_is_the_same_run() {
    in_its_own_process(|| {
        let first = launched("th06nc-twice-a");
        played_out(&first);
        let one = the_world_each_frame(&first);
        assert!(
            !one.is_empty(),
            "the run wrote no status line to hold against another:\n  {}",
            first.log().lines().join("\n  ")
        );
        drop(first);

        let second = launched("th06nc-twice-b");
        played_out(&second);
        let two = the_world_each_frame(&second);

        assert_eq!(
            one.len(),
            two.len(),
            "the two runs are different lengths, so the fake answers a different world from the same \
             seed: {} against {}",
            one.len(),
            two.len(),
        );
        // The first line they differ on, rather than the whole of both: what a divergence needs saying
        // about it is where it began.
        let parted = one.iter().zip(&two).position(|(a, b)| a != b);
        assert_eq!(
            parted,
            None,
            "the two runs parted at line {}:\n  one: {}\n  two: {}",
            parted.unwrap_or(0),
            one.get(parted.unwrap_or(0)).map_or("", String::as_str),
            two.get(parted.unwrap_or(0)).map_or("", String::as_str),
        );
    });
}
