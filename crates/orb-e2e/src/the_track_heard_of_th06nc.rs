//! **Which track th06nc is playing, in the log, once per change of it.**
//!
//! What `the_track_heard_of_th06c.rs` holds that game to, held to this one, whose `Th06nc::music_identity` is
//! the same kind of reading at its own address: the track number the game keeps in `THE_TRACK_PLAYING`, no
//! name beside it, and — `Th06nc::music` answering `None` — no position either.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth.
const APART: u32 = 60;

/// Two tracks a run plays, by the numbers the game keeps them under.
const STAGE_SONG: i32 = 2;
const FIGHT_SONG: i32 = 3;

/// How many lines of the log say `said`.
fn lines_saying(game: &Fake, said: &str) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| line.contains(said))
        .count()
}

#[test]
fn the_track_th06nc_plays_is_named_in_the_log_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-the-track-heard",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        game.fires_at_the_player(false);
        game.in_a_run_in_the_stage(0);
        game.frames(APART);

        let stage_song = format!("sound: {:#010x} is ", STAGE_SONG);
        let fight_song = format!("sound: {:#010x} is ", FIGHT_SONG);
        game.image().plays_the_track(STAGE_SONG);
        game.frames(APART);
        assert_eq!(
            lines_saying(&game, &stage_song),
            1,
            "the stage's track was not said once:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.log().said(&format!(
                "{stage_song}on the stream, whether it plays and where not to be had"
            )),
            "the line claims a state or a position this game has no reading of:\n  {}",
            game.log().lines().join("\n  ")
        );

        game.image().plays_the_track(FIGHT_SONG);
        game.frames(APART);
        assert_eq!(
            lines_saying(&game, &fight_song),
            1,
            "the change to the fight's track was not said once:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            lines_saying(&game, &stage_song),
            1,
            "the stage's track was said again after it had gone",
        );

        let from = game.log().written();
        game.image().plays_the_track(-1);
        game.frames(APART);
        assert!(
            game.log().said_since(from, "sound: no track is streaming"),
            "the track going was not said:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
