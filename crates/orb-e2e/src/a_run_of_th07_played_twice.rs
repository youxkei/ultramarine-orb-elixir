//! **Whether a th07 laid out by hand answers the same world twice from the same seed and the same buttons.**
//!
//! Which is the question under every claim about putting a run back: a playback feeds the file's buttons to
//! the game's own updates and expects the world the run had, and that expectation is only meaningful if the
//! game *has* one world for a given seed and a given sequence of buttons. A fake that answered differently
//! twice would make reproduction untestable here whatever orb did, and the real game would be the only
//! authority for it — `a_run_of_th08_played_twice` is the same question of that game's fake.
//!
//! **No chance in the recipe**, which is what makes the two runs comparable at all: a `take_hits` is a draw,
//! so a recipe carrying one is two different runs by design. What is left is the walk and a `play_until`,
//! both of which are the same frames every time — through the laid-out stage's midboss and her card, which
//! is what the run's third chapter is.

use crate::fake::th07::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out.
const FRAMES: u32 = 30_000;

/// How long a stage of the laid-out game lasts here, as the real game's cases are given one — past the
/// midboss's fight, whose card is the third chapter's start.
const A_STAGE: u32 = 6_000;

/// The recipe both launches play: the walk to a run, the run played into its third chapter, and out.
fn twice() -> String {
    format!(
        "{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    )
}

/// A launch of the game as the real game's cases launch it — its stage's own track streaming, a shot select
/// that starts the run, and records that light nothing — so that the two differ in nothing but their names.
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
    game.streams_its_song(0);
    game.its_shot_select_starts_a_run_however_it_came_up();
    game.stages_last(A_STAGE);
    game.its_records_hold_no_clear();
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
fn a_run_of_th07_played_twice_from_the_same_seed_is_the_same_run() {
    in_its_own_process(|| {
        let first = launched("th07-twice-a");
        played_out(&first);
        let log = first.log();
        assert!(
            log.said("stage 1 chapter 3"),
            "the run never reached the chapter its recipe plays into:\n  {}",
            log.lines().join("\n  ")
        );
        let one = the_world_each_frame(&first);
        assert!(
            !one.is_empty(),
            "the run wrote no status line to hold against another:\n  {}",
            log.lines().join("\n  ")
        );
        drop(first);

        let second = launched("th07-twice-b");
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
