//! **What the music coming out of th08 is, in the log, once per change of it.**
//!
//! What `the_track_heard_of_th07.rs` asks of that game, asked of this one: every other line about the sound is
//! a position, a cursor or a countdown, and none of them says which sound those numbers belong to. This game
//! keeps the name of the track it streams on the record the stream reads, as th07 does — see
//! `Th08::the_track_heard` — so the line names it by the file's own name for it.
//!
//! **What the line says beside the name is not asked here.** `Th08::music` answers nothing, the stream being
//! left out of every restore rather than put back — see its own doc — so what the runtime can say of whether
//! the track is playing, and where, is that it has neither to give — `the_track_heard_of_th06c.rs` holds that
//! wording; the name is the half this game has.
//!
//! At `normal`, and once per change: what it is for is a run played by a recipe, where the check on the music
//! is the names in the log held against the stages the run walked through.

use crate::fake::th08::{Fake, THE_FIGHTS_TRACK};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth: what this
/// reads is that the line is said once for a track and not once a frame.
const APART: u32 = 60;

/// The two tracks a stage plays, by the names the fake's own `thbgm.dat` records carry.
const STAGE_SONG: &str = "th08_01.wav";
const FIGHT_SONG: &str = "th08_02.wav";

/// The lines that name `track` as the one being heard.
fn naming(game: &Fake, track: &str) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| line.contains(&format!("sound: {track} ")))
        .count()
}

#[test]
fn the_log_of_th08_names_the_track_being_heard_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-track-heard",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        let log = game.log();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames(APART);
        assert_eq!(
            naming(&game, STAGE_SONG),
            1,
            "the stage's own song was not named once as what is being heard:\n  {}",
            log.lines().join("\n  "),
        );

        // The fight the stage ends with, which is the other song the stage's info block names: a track
        // change, and the one thing a line about the sound has to follow.
        game.image().the_stream_reads_the_track(THE_FIGHTS_TRACK);
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
    });
}
