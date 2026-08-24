//! **An ending of th07 run out inside one frame, and its staff roll left to play.**
//!
//! What `the_ending.rs` asks of th06, asked through this game's own two fields. An ending here is a scene and
//! an object hung off the chain — the register keeps its address nowhere else, so a walk of the chain for the
//! ending's own update is the whole of how orb finds one — and the script that object is reading is the one
//! thing that says where a part of an ending ends: th07 moves on by reading the next file over the one it was
//! running, and neither the flag nor the scene moves with it. So the roll is the ending's last script and
//! nothing else marks where it begins.
//!
//! **Which is what the run-out has to stop at.** An ending is minutes of waits and orb runs it out inside the
//! frame it begins on — the game's own update called again and again with nothing drawn between them — and
//! stopping anywhere but the roll would either leave minutes of it to sit through or run out the one part of an
//! ending that was worth keeping.
//!
//! How long any of it took is one machine's and is not written down. The updates are the game's own.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many updates the ending's own script takes before it hands over to the roll's: a number this file
/// chooses, since what a part of an ending is worth of waits is the game's own data and what is being asked here
/// is that orb spends them all inside one frame.
const ENDING_UPDATES: i32 = 12_000;

/// And how many the other one runs before its scene changes under it: a handful, that arm being about where the
/// loop stops rather than about how much it can spend.
const UPDATES_BEFORE_THE_SCENE_GOES: i32 = 20;

/// A launch with an ending running, taken to the frame orb runs it out in.
fn into_an_ending(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

#[test]
fn an_ending_of_th07_is_run_out_in_one_frame_and_stops_where_its_staff_roll_begins() {
    in_its_own_process(|| {
        let game = into_an_ending("th07-the-ending");
        let log = game.log();
        // A track under it, so that the roll starting one of its own is there to be read back: the ending plays
        // one and the roll another, which is the second thing saying where one ends and the other begins.
        game.streams_its_song(0);
        game.an_ending_is_running(ENDING_UPDATES);

        // One frame, and the whole of the ending's own script is spent in it.
        game.one_frame();
        let said = log
            .lines()
            .into_iter()
            .find(|line| line.contains("ending run out in"))
            .unwrap_or_else(|| {
                panic!(
                    "the ending was not run out on the frame it began on:\n  {}",
                    log.lines().join("\n  ")
                )
            });
        // Every update of that script but the first, which is the frame's own: orb's frame runs the game's update
        // once and then finds an ending in front of it, so what the line counts is the updates the run-out spent
        // itself.
        assert!(
            said.contains(&format!(
                "ending run out in {} update(s)",
                ENDING_UPDATES - 1
            )),
            "the updates the ending took are not the ones its script had: {said}",
        );
        assert!(
            said.contains("where its staff roll begins"),
            "the run-out stopped somewhere other than the roll: {said}",
        );
        // And the two tracks named, which is what says the roll's own started: an ending run out to a track that
        // never changed is one the run-out went past the handover on.
        let tracks: Vec<&str> = said
            .rsplit("track ")
            .next()
            .expect("the track the line ends with")
            .split(" -> ")
            .collect();
        assert_eq!(tracks.len(), 2, "the line does not name two tracks: {said}");
        assert_ne!(
            tracks[0], tracks[1],
            "the roll began under the ending's own track: {said}",
        );

        // The roll is then the game's to play, a frame at a time: the flag stays set and the scene stays the
        // ending's, so a second run-out would spend the whole roll in one frame.
        let after = log.written();
        game.frames(60);
        assert!(
            !log.said_since(after, "ending run out in"),
            "the staff roll was run out as well:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.state().in_ending,
            "the roll is not inside the ending as far as the game is concerned: {}",
            game.state(),
        );
    });
}

/// And an ending whose scene changes under it: the run-out stops there too, so that whatever follows an ending
/// is reached and then left alone.
///
/// Which is the arm that keeps the loop bounded by something the game does rather than by the ending's own
/// script: a part that hands over to nothing would otherwise be run to the limit.
#[test]
fn an_ending_of_th07_that_leaves_its_scene_is_left_where_it_went() {
    in_its_own_process(|| {
        let game = into_an_ending("th07-the-ending-left");
        let log = game.log();
        // An ending whose script never hands over: what stops the run-out is the scene changing under it, a few
        // updates in.
        game.an_ending_that_leaves_its_scene_is_running(UPDATES_BEFORE_THE_SCENE_GOES);
        game.one_frame();

        let said = log
            .lines()
            .into_iter()
            .find(|line| line.contains("ending skipped,"))
            .unwrap_or_else(|| {
                panic!(
                    "the ending that left its scene was not said out loud:\n  {}",
                    log.lines().join("\n  ")
                )
            });
        assert!(
            said.contains("scene 9 -> 1"),
            "the line does not say which scene the ending went to: {said}",
        );
        assert!(
            !game.state().in_ending,
            "the ending is still running: {}",
            game.state(),
        );
        // And what it went to is left alone: the front end is up and orb ran no more of it out.
        let after = log.written();
        game.frames(60);
        assert!(
            !log.said_since(after, "ending skipped,")
                && !log.said_since(after, "ending run out in"),
            "orb ran the scene after the ending out as well:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
