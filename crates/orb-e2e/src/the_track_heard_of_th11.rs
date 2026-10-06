//! **Which track th11 is playing, in the log, once per change of it.**
//!
//! What `the_track_heard_of_th10.rs` asks of that game, asked of this one: every other line about the sound is
//! a position, a cursor or a countdown, and none of them says which sound those numbers belong to. This game
//! keeps the record of `thbgm.fmt` it streams on the stream's source — its address at the source's `+0x90`
//! (0x45b31d), where th10's held the record's index — see `Th11::the_track_heard` — so the line names it by the
//! file's own name for it.
//!
//! **And nothing of where in the track it is**, `Th11::music` answering `None`: the stream is left out of every
//! restore rather than put back. So what the line carries past the name is that neither whether it plays nor
//! where is to be had — `the_track_heard_of_th06c.rs`'s wording.
//!
//! At `normal`, and once per change: what it is for is a run played by a recipe, where the check on the music
//! is the names in the log held against the stages the run walked through.

use crate::fake::th11::{Fake, THE_BOSSS_TRACK, THE_STAGES_TRACK};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a track is left playing before the next one, which is more than one line's worth: what this
/// reads is that the line is said once for a track and not once a frame.
const APART: u32 = 60;

/// The two tracks a stage plays, by the names the fake's own `thbgm.fmt` records carry: stage 1's own two.
const STAGE_SONG: &str = "th11_01.wav";
const FIGHT_SONG: &str = "th11_02.wav";

/// The frames the stage is given to begin and ask for its song.
const FRAMES_FOR_A_BUILD: u32 = 240;

/// The lines that name `track` as the one being heard.
fn naming(game: &Fake, track: &str) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| line.contains(&format!("sound: {track} ")))
        .count()
}

/// **A stage's song is a command in the sound's queue and not a play made there and then**: the stage's build
/// loads the stage's two songs' paths through 0x42a0d0, command 1, and waits until the queue `[0x4c5e08]` is empty
/// (0x41fd87 through 0x41fd9e), so loading them plays nothing; the stage begun asks for the song through 0x42a150,
/// which queues command 2 (0x42a17e), and 0x44a340 carries a command 2 out a step a call — the source put on the
/// record at step 2, its third call (0x44a690), and the command off the queue at step 7, its eighth.
#[test]
fn a_stage_of_th11_queues_its_song_and_the_sound_plays_it_calls_later() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-song-queued",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.image().the_stream_plays_the_track(THE_BOSSS_TRACK);
        // The run placed, its build run out — the songs' paths loaded and out of the queue — before the stage's
        // first frame.
        game.is_in_a_run();
        assert_eq!(
            game.image().the_track_the_stream_is_on(),
            Some(THE_BOSSS_TRACK),
            "loading the songs' paths played one of them",
        );
        game.frames_until("the stage's song asked for", FRAMES_FOR_A_BUILD, || {
            game.image().the_sound_queue_holds_a_command()
        });
        assert_eq!(
            game.image().the_track_the_stream_is_on(),
            Some(THE_BOSSS_TRACK),
            "the stage's song was played on the run it was asked for",
        );
        let mut runs = 0;
        while game.image().the_track_the_stream_is_on() != Some(THE_STAGES_TRACK) {
            game.frame();
            runs += 1;
            assert!(runs <= A_PLAY_TAKES, "the stage's song never came on");
        }
        assert!(
            runs >= A_PLAY_IS_HEARD_AFTER,
            "the stage's song came on {runs} run(s) after it was asked for, before the play's step 2",
        );
        assert!(
            game.image().the_sound_queue_holds_a_command(),
            "the play left the queue on the run it was heard on",
        );
        game.frames_until("the play out of the queue", A_PLAY_TAKES, || {
            !game.image().the_sound_queue_holds_a_command()
        });
    });
}

/// The runs a play takes to be heard after the one it was asked on, and to leave the queue: step 2 is the third
/// call, and step 7 the eighth.
const A_PLAY_IS_HEARD_AFTER: u32 = 3;
const A_PLAY_TAKES: u32 = 8;

#[test]
fn the_log_of_th11_names_the_track_being_heard_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-track-heard",
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

        // The boss's own song, which the conversation before her puts on (0x41e9cf through 0x41e9dd): a track
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
