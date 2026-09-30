//! **Whether a th10 laid out by hand answers the same world twice from the same seed and the same buttons.**
//!
//! Which is the question under every claim about putting a run back: a playback feeds the file's buttons to
//! the game's own updates and expects the world the run had, and that expectation is only meaningful if the
//! game *has* one world for a given seed and a given sequence of buttons. A fake that answered differently
//! twice would make reproduction untestable here whatever orb did, and the real game would be the only
//! authority for it — `a_run_of_th08_played_twice` is the same question of that game's fake.
//!
//! **No chance in the recipe**, which is what makes the two runs comparable at all: a `take_hits` is a draw,
//! so a recipe carrying one is two different runs by design. What is left is the walk and a `play_until`,
//! both of which are the same frames every time.

use crate::fake::th10::{Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long a stage of the laid-out game lasts here: past its midboss, whose card from frame 1500 is the
/// stage's third chapter, and on to its boss.
const A_STAGE: u32 = 4_500;

/// The recipe both launches play: the walk to a run, the run played into its third chapter, and out.
fn twice() -> String {
    format!(
        "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    )
}

fn launched(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(twice());
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
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
    let lines = game.log().lines();
    panic!(
        "the recipe never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  ")
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
fn a_run_of_th10_played_twice_from_the_same_seed_is_the_same_run() {
    in_its_own_process(|| {
        let first = launched("th10-twice-a");
        played_out(&first);
        let one = the_world_each_frame(&first);
        assert!(
            !one.is_empty(),
            "the run wrote no status line to hold against another:\n  {}",
            first.log().lines().join("\n  ")
        );
        drop(first);

        let second = launched("th10-twice-b");
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
