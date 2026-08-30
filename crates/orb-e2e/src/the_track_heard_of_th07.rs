//! **What the music coming out is, in the log, once per change of it.**
//!
//! Every other line about the sound is a position, a cursor or a countdown, and none of them says which sound
//! those numbers belong to: a launch that put one track's position into a stream playing another said so in
//! numbers a reader cannot tell from the right ones. A th07 session picked stage 6's first chapter up and
//! heard the fifth stage's boss theme, and the whole of what its log had was `music: the track loops at
//! 16545584, so 0 byte(s) left from 242508064`.
//!
//! **Named for th07, which keeps the name of the track it streams** on the record the stream reads — see
//! `Th07::the_track_playing`. th06 keeps no name and its own line tells one track from another by the
//! identity instead; `the_track_heard.rs` is that half.
//!
//! At `normal`, and once per change: what it is for is a run played by a recipe, where the check on the music
//! is the names in the log held against the stages the run walked through.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth: what this
/// reads is that the line is said once for a track and not once a frame.
const APART: u32 = 60;

/// The two tracks a stage plays, by the names the fake's own `thbgm.dat` records carry.
const STAGE_SONG: &str = "th07_02.wav";
const FIGHT_SONG: &str = "th07_03.wav";

/// **And the stop said from inside the game's own stop of the music.**
///
/// Which is the other half of where a sound is made, and the half no reading from the frame can date: a buffer
/// stopped between two frames is stopped by the time anything asks, with nothing saying when or at whose ask.
/// The pause menu's own pause is one of those asks — the manager sends it on the frame that menu goes up, and
/// what carries it out is the stream's own `Stop`.
#[test]
fn the_stop_of_th07s_music_is_said_from_inside_the_games_own_stop() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-stop",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);

        // The pause menu up, which is the manager's own ask: the menu button's edge, and then the frames its
        // command is carried out on.
        let from = log.written();
        game.press(keys::ESCAPE);
        game.frames(4);
        assert!(
            log.said_since(from, "sound: stopped through the game"),
            "nothing was said from inside the stream's own stop, so the pause the menu asked for has no \
             line to be dated by:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// The countdown a refill's own line carries: what is left of the track before the loop is taken.
///
/// # Panics
/// Where the line is not one of the refill's, which is a test asserting on a line it did not mean.
fn countdown_in(line: &str) -> u32 {
    line.rsplit(", ")
        .next()
        .and_then(|tail| tail.split(' ').next())
        .and_then(|number| number.parse().ok())
        .expect("a refill's own line, whose last field is the countdown")
}

/// **And what the game's own refill is feeding the buffer, once per refill.**
///
/// Which is the reading no line taken from the frame can stand in for: a track looping over a section of
/// itself leaves the same track playing, its buffer fed and its position wherever the last refill put it, so
/// everything read from outside says the music is fine. What the refill says is the countdown before the loop,
/// and a section loop is that number turning over a narrow range instead of climbing down to nothing.
///
/// **The turning over is read off these lines and is not arranged here**: the fake's own stream only ever
/// moves forward, and what a test of it would need is a stream put back mid-track, which is a change to the
/// fake rather than to the hook. What this holds is that each refill says its own numbers.
#[test]
fn the_refill_of_th07_says_what_it_is_feeding_the_buffer() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-refill",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        let before = game.the_countdown();
        game.services_the_buffer();
        game.services_the_buffer();
        let said = log
            .lines()
            .iter()
            .filter(|line| line.contains("sound: the refill"))
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            said.len(),
            2,
            "the refill said other than one line per refill:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            countdown_in(&said[0]),
            before,
            "the first refill did not say the countdown the stream was on: {}",
            said[0],
        );
        assert!(
            countdown_in(&said[1]) < countdown_in(&said[0]),
            "the countdown did not move between the two refills, so the lines say nothing about whether \
             the track is advancing or going round a section of itself:\n  {}\n  {}",
            said[0],
            said[1],
        );
    });
}

#[test]
fn the_log_of_th07_names_the_track_being_heard_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-track-heard",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);

        assert!(
            log.said(&format!("sound: {STAGE_SONG} is playing")),
            "the log does not say the stage's own song is what is being heard:\n  {}",
            log.lines().join("\n  "),
        );
        let of_the_stage = log
            .lines()
            .iter()
            .filter(|line| line.contains(&format!("sound: {STAGE_SONG}")))
            .count();
        assert_eq!(
            of_the_stage,
            1,
            "the track being heard was said more than once for one track:\n  {}",
            log.lines().join("\n  "),
        );

        // The fight the stage ends with, which is the other song the stage's data names: a track change, and
        // the one thing a line about the sound has to follow.
        game.the_stages_own_boss_arrives();
        game.frames(APART);
        assert!(
            log.said(&format!("sound: {FIGHT_SONG} is playing")),
            "the log did not follow the track to the fight's own song:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
