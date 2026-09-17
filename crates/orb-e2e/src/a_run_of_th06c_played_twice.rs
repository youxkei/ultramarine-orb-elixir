//! **Whether a th06c laid out by hand answers the same world twice from the same seed and the same buttons.**
//!
//! Which is the question under every claim about putting a run back: a playback feeds the file's buttons to
//! the game's own updates and expects the world the run had, and that expectation is only meaningful if the
//! game *has* one world for a given seed and a given sequence of buttons. A fake that answered differently
//! twice would make reproduction untestable here whatever orb did, and the real game would be the only
//! authority for it.
//!
//! **This exists because a playback of this game came out of step and neither cause could be ruled out.**
//! Two launches of the same recipe, and the status lines the runtime writes each frame held against each
//! other: the same lines, and the fake is deterministic and the divergence is orb's to find; different
//! lines, and it is the fake's. **They matched**, and the divergence was orb's twice over — the word the
//! frame that builds a resumed stage was played with, and a snapshot waiting for music this game has none
//! of. `a_pointdevice_run_of_th06c`'s pick-up test is what asks about the playback now, frame by frame.
//!
//! **No rules in the recipe**, which is what makes the two runs comparable at all: `take_hits` and the
//! steering are a chance apiece, so a recipe carrying them is two different runs by design. What is left is
//! the walk and `play_until`, both of which are the same frames every time.

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// The recipe both launches play: the walk to a run, the run played into its third chapter, and out.
fn twice() -> String {
    format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 3 }}
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

/// The run's own world, frame by frame, as the runtime's status line carries it — the timestamp cut off, that
/// being the one part of the line no two launches share.
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
fn a_run_of_th06c_played_twice_from_the_same_seed_is_the_same_run() {
    in_its_own_process(|| {
        let first = launched("th06c-twice-a");
        played_out(&first);
        let one = the_world_each_frame(&first);
        assert!(
            !one.is_empty(),
            "the run wrote no status line to hold against another:\n  {}",
            first.log().lines().join("\n  ")
        );
        drop(first);

        let second = launched("th06c-twice-b");
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
        // The first line they differ on, rather than the whole of both: what a divergence needs saying about
        // it is where it began.
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
