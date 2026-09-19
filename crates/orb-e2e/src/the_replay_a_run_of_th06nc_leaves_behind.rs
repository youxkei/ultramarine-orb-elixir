//! **What a Pointdevice run of th06nc leaves for somebody to watch: every stretch it played, the ones that
//! were rewound away included.**
//!
//! What `the_replay_a_run_leaves_behind.rs` asks of th06, `the_replay_a_run_of_th07_leaves_behind.rs` of
//! th07 and `the_replay_a_run_of_th06c_leaves_behind.rs` of the other 64-bit build, asked of this game. The
//! replay is orb's own and one shape for all four — this game records none of its own that would survive,
//! its own recorder being a thing a chapter's restore rewinds — so what is this game's is the run it is
//! written out of: the frame counter of the run, the chapters its fight is divided into, and the slot its
//! three numbers name.
//!
//! **A run still being played is not a saved replay**, and the extension is what says so: `.pending` is the
//! run somebody is in the middle of and `.msgpack` is one that has been kept.
//!
//! **Walked into through the front end and not said**, which is the one thing these need that the other e2e
//! tests over this game do not: the file is opened where the run is *chosen* — the frame the front end's own
//! decide starts one — so a run put in place by writing the game's memory has no replay being written at all.

use std::path::PathBuf;

use orb_config::LogLevel;

use crate::fake::th06nc::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// Where orb keeps them, and what it calls the two kinds.
///
/// Written out here rather than taken from `replay`: what an e2e test says orb left behind is a file at a
/// path, and one that borrowed orb's own idea of that path could not fail if orb changed it.
const KEPT_IN: &str = "pointdevice_replay";
const WHILE_PLAYING: &str = "pending";
const ONCE_KEPT: &str = "msgpack";

/// The name orb gives the run these play, which names its file: Normal, and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 60_000;

/// **A run whose first attempt at a chapter is hit on purpose and whose second is played clean**, which is
/// the recipe `a_death_answered_by_the_chapter_again` plays on the real game: the window of frames is where
/// the one death falls, the chapter it falls in being a thing of how well the rules play.
fn a_run_rewound_once(name: &str) -> Box<Fake> {
    let recipe = format!(
        "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ stage: 1, chapter: 4 }}
  give_up_after: 36000
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
    let log = game.log();
    game.frames_until("an attempt rewound away", FRAMES, || {
        log.said("was rewound away after")
    });
    game
}

/// Every file orb left under its own replay directory.
fn replays(game: &Fake) -> Vec<String> {
    game.sim()
        .files()
        .paths()
        .into_iter()
        .map(|path: PathBuf| path.to_string_lossy().into_owned())
        .filter(|path| path.contains(KEPT_IN))
        .collect()
}

/// How many attempts the file orb is writing holds, off the line it writes every time it writes one.
///
/// # Panics
/// If no such line has been written, which is a run whose replay is not being written at all.
fn attempts_in_the_file(game: &Fake) -> u32 {
    let said = game
        .log()
        .lines()
        .into_iter()
        .rev()
        .find(|line| line.contains(" attempt(s) written to "))
        .expect("a line saying what orb wrote into the run's replay");
    said.split(" attempt(s) written to ")
        .next()
        .and_then(|head| head.rsplit(", ").next())
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("no count of attempts in {said}"))
}

/// **A run rewound from a chapter leaves the attempt it was rewound from behind as well as the one it went on
/// with**, which is the whole of what a Pointdevice replay is for: a recorder of the game's own would keep
/// the path that survived and nothing of the tries.
#[test]
fn a_run_of_th06nc_writes_down_the_attempt_it_was_rewound_from_as_well_as_the_one_it_went_on_with()
{
    in_its_own_process(|| {
        let game = a_run_rewound_once("th06nc-the-replay-two-attempts");
        let rewound = attempts_in_the_file(&game);
        assert!(
            rewound >= 2,
            "the run was rewound from a chapter with {rewound} attempt(s) in its file, which is no try \
             and the one it was rewound from",
        );

        // And the run played on out of it, which is the attempt the retry began: what stands in the file
        // afterwards is more than what the rewind left.
        for _ in 0..FRAMES {
            if game.sim().exited().is_some() {
                break;
            }
            game.frame();
        }
        assert!(
            attempts_in_the_file(&game) > rewound,
            "the attempt the retry began was never written down: {rewound} both times",
        );
    });
}

/// And the file is the run's own, under orb's own directory, and **pending for as long as the run is being
/// played**.
#[test]
fn the_replay_of_a_run_of_th06nc_is_pending_while_it_is_being_played() {
    in_its_own_process(|| {
        let game = a_run_rewound_once("th06nc-the-replay-pending");
        let pending = format!("{KEPT_IN}\\{SLOT}.{WHILE_PLAYING}");
        let left = replays(&game);
        assert!(
            left.iter().any(|path| path.ends_with(&pending)),
            "the run being played left no {pending}: {left:?}",
        );
        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| line.contains("replay:"))
            .collect();
        assert!(
            !left.iter().any(|path| path.ends_with(ONCE_KEPT)),
            "a run still being played was kept all the same: {left:?}\n  {}",
            said.join("\n  ")
        );

        // Played out, which ends with the run given up at orb's own menu.
        for _ in 0..FRAMES {
            if game.sim().exited().is_some() {
                break;
            }
            game.frame();
        }
        assert!(
            game.log()
                .said("retry: the run is given up; the game is on its way to the title"),
            "the recipe did not give the run up:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
