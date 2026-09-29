//! **Which track th06c is playing, in the log, once per change of it.**
//!
//! What `the_track_heard_of_th07.rs` holds th07 to, held to this game: every other line about the sound is a
//! position, a cursor or a countdown, and none of them says which sound those numbers belong to. **What this
//! game gives the line is the identity alone** — `Th06c::music_identity` is the track number the game keeps in
//! `THE_TRACK_PLAYING`, and it keeps no name — so a track is told from another by that number, the way th06's
//! own line tells them apart.
//!
//! **And nothing of where in the track it is**, `Th06c::music` answering `None`: the track is in a block every
//! restore leaves where it finds it, and its position is not read. So what the line carries past the number
//! is that neither whether it plays nor where is to be had — no `stopped` over a track this game is playing,
//! which is what `audio::what_is_heard` said of every one before it asked whether it had a buffer to read.
//!
//! At `normal`, and once per change: what it is for is a run played by a recipe, where the check on the music
//! is the tracks in the log held against the stages the run walked through.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth: what this
/// reads is that the line is said once for a track and not once a frame.
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
fn the_track_th06c_plays_is_named_in_the_log_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-track-heard",
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

        // And the music gone, which is a change as well: the game's own `-1`.
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
