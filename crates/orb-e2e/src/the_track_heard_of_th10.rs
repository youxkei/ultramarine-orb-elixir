//! **Which track th10 is playing, in the log, once per change of it.**
//!
//! What `the_track_heard_of_th08.rs` asks of that game, asked of this one: every other line about the sound is
//! a position, a cursor or a countdown, and none of them says which sound those numbers belong to. This game
//! keeps the name of the track it streams on the record of `thbgm.fmt` the stream reads, as th08 does — see
//! `Th10::the_track_heard` — so the line names it by the file's own name for it.
//!
//! **And nothing of where in the track it is**, `Th10::music` answering `None`: the stream is left out of every
//! restore rather than put back. So what the line carries past the name is that neither whether it plays nor
//! where is to be had — `the_track_heard_of_th06c.rs`'s wording.
//!
//! At `normal`, and once per change: what it is for is a run played by a recipe, where the check on the music
//! is the names in the log held against the stages the run walked through.

use crate::fake::th10::{Fake, THE_BOSSS_TRACK, THE_STAGES_TRACK};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth: what this
/// reads is that the line is said once for a track and not once a frame.
const APART: u32 = 60;

/// The two tracks a stage plays, by the names the fake's own `thbgm.fmt` records carry.
const STAGE_SONG: &str = "th10_00.wav";
const FIGHT_SONG: &str = "th10_01.wav";

/// The lines that name `track` as the one being heard.
fn naming(game: &Fake, track: &str) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| line.contains(&format!("sound: {track} ")))
        .count()
}

#[test]
fn the_log_of_th10_names_the_track_being_heard_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-track-heard",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().the_stream_plays_the_track(THE_STAGES_TRACK);
        game.frames(APART);
        assert_eq!(
            naming(&game, STAGE_SONG),
            1,
            "the stage's own song was not named once as what is being heard:\n  {}",
            log.lines().join("\n  "),
        );

        // The boss's own song, which the conversation before her puts on (0x416a91 through 0x416a9c): a track
        // change, and the one thing a line about the sound has to follow.
        game.image().the_stream_plays_the_track(THE_BOSSS_TRACK);
        game.frames(APART);
        assert_eq!(
            naming(&game, FIGHT_SONG),
            1,
            "the log did not follow the track to the fight's own song, once:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            naming(&game, STAGE_SONG),
            1,
            "the stage's own song was named again with nothing changed back to it:\n  {}",
            log.lines().join("\n  "),
        );

        // And the music gone, which is a change as well: the manager with no stream.
        let from = log.written();
        game.image().the_sound_is_not_up();
        game.frames(APART);
        assert!(
            log.said_since(from, "sound: no track is streaming"),
            "the track going was not said:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
