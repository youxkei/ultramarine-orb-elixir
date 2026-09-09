//! **The music of th07 put back where a chapter had it, out of the file the game opens itself.**
//!
//! What is being asked is the same thing `the_music_across_a_restore.rs` asks of th06 — the buffer's bytes,
//! the mixer's cursor and how far the file has been read, none of which is memory and so none of which a
//! snapshot holds — through the half of it that is *not* the same: th06 opens a wav per track through
//! winmm's `mmio`, and th07 opens `thbgm.dat` once itself and reads every track out of that handle. So a
//! position here is an absolute offset in the archive, moved with `SetFilePointer` and read with `ReadFile`,
//! which is `orb_core::audio::Track`'s second shape.
//!
//! The stream is a real buffer and those two are real functions, because orb reaches them by calling rather
//! than by reading — see `orb_sim::Sound`. What no e2e test can say is what any of it sounds like; the numbers
//! below stand in for that.

use crate::fake::th07::{Fake, STREAM_BUFFER, STREAM_NOTIFY, TRACK};
use crate::fake::{Display, Launched, PRESENT, SOUND, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th07::Th07;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How far into the track the sound now audible begins when the chapter is taken: part way in, so that a
/// position put back to the beginning of the file is a different number from the one that was there.
const HEARD_AT: i32 = 0x100;

/// **A stream whose buffer is no longer a live object is a stage whose chapters hold no sound.**
///
/// The word at the buffer's head is the whole of what orb has to tell a live COM object from the stale pointer
/// a released one left behind, and it looks at it because the alternative is calling through that pointer: the
/// game releases the buffer the moment it changes track, and a snapshot that copied the object back would be
/// putting a freed one under the streaming thread.
///
/// So `Th07::music` answers nothing, and the whole of what follows is that: the stage waits out the whole of the
/// wait for a track that will never be readable, takes its first chapter without sound, and plays on. Which is
/// worth holding orb to because the run is still a run — the chapters rewind, the retry menu works, and only
/// the music is gone.
#[test]
fn a_stream_of_th07_whose_buffer_has_been_freed_leaves_the_chapters_without_sound() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-freed",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // A real buffer first, so that what the next line does is take a live one away rather than never lay
        // one out: a stage with no stream at all is a different launch.
        game.streams_its_song(HEARD_AT);
        assert!(
            Game::music(&Th07).is_some(),
            "the stream was not readable before this e2e test freed its buffer",
        );
        game.image().frees_the_stream_buffer();
        assert!(
            Game::music(&Th07).is_none(),
            "orb still reads a stream whose buffer is not a live object",
        );

        // The run goes as it always does, and its first chapter is taken with no sound in it.
        game.in_a_pointdevice_run();
        assert_eq!(
            log.lines()
                .iter()
                .filter(|line| line.contains("chapter 1 (stage start)")
                    && line.contains("music: false"))
                .count(),
            1,
            "the stage's first chapter reports sound it cannot have read:\n  {}",
            log.lines().join("\n  ")
        );

        // And the run is still a run: the hit, チャプターをやり直す, and the chapter comes back.
        let at_the_chapter = game.state();
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_chapter,
            "a chapter with no sound in it did not come back",
        );
    });
}

#[test]
fn a_chapter_of_th07_is_put_back_with_the_stream_the_music_was_at() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        // A run with its stage's track streaming, which is what `SoundPlayer::PlayBGM` leaves: the stream, the
        // source reading `thbgm.dat`, and the record of the track that source is on.
        game.streams_its_song(HEARD_AT);
        game.in_a_pointdevice_run();

        // The chapter the stage's start is, taken with the music in it — `music: true` is that chapter's own
        // line saying the capture came off, and a chapter taken with nothing of its music is one this test
        // would say nothing about.
        assert!(
            log.said("music: true"),
            "the chapter was taken with nothing of its music in it:\n  {}",
            log.lines().join("\n  ")
        );
        let at_the_chapter = game.stream_now();
        let bookkeeping = (game.image().next_write_offset(), game.image().bytes_left());
        assert_eq!(
            at_the_chapter.position, HEARD_AT,
            "the file is not where this test put it",
        );

        // And then the streaming thread runs: two chunks read out of the file into the buffer, and the mixer
        // part way through what was already in there. Every one of the four moves.
        game.services_the_buffer();
        game.services_the_buffer();
        game.plays_the_buffer_on(STREAM_NOTIFY + STREAM_NOTIFY / 2);
        let moved_on = game.stream_now();
        assert_ne!(
            moved_on, at_the_chapter,
            "the stream did not move, so nothing here says it came back",
        );
        assert_ne!(
            (game.image().next_write_offset(), game.image().bytes_left()),
            bookkeeping,
            "the game's own streaming fields did not move with it",
        );

        // The hit and チャプターをやり直す.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // The stream is the one the chapter was taken with, byte for byte — through the same buffer and the
        // same handle, which is what says the sound was put back rather than taken down and started again.
        assert_eq!(
            game.stream_now(),
            at_the_chapter,
            "the sound did not come back where the chapter had it",
        );
        assert!(
            !log.said("the track has changed since this snapshot"),
            "the track the chapter was taken with was not recognised as the one playing:\n  {}",
            log.lines().join("\n  ")
        );

        // And the game's own bookkeeping came back with the rest of its memory, which is the half of the pair
        // that has to agree with the buffer: the offset the next chunk goes at, and the countdown the track's
        // loop is taken on.
        assert_eq!(
            (game.image().next_write_offset(), game.image().bytes_left()),
            bookkeeping,
            "the streaming fields the game keeps did not come back with the buffer",
        );
    });
}

/// And a track the game has changed since the chapter was taken is not one a restore puts a buffer back into:
/// the record the source is reading is what says which track it is, so the music is taken down and the
/// chapter's own track started again instead.
///
/// **What comes back is read as a track and not as a slot**, which is the whole of what this asks that the
/// count of the calls did not. The game is asked for a slot, a slot means the name loaded into it, and a name
/// means one of `thbgm.dat`'s records — so a slot nothing was ever loaded into is not silence but the file's
/// first record, the title screen's song. That is what a th07 session heard where the chapter's own track was
/// asked for by a fixed number: the music never came back and the title's began instead.
#[test]
fn a_chapter_of_th07_taken_under_another_track_gets_its_own_track_back() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-changed",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(HEARD_AT);
        game.in_a_pointdevice_run();
        assert!(log.said("music: true"));

        // The boss's own music, which is another record in the same file: the game loads it as the fight
        // starts, and the stream the chapter was taken with is gone with the track it was playing.
        game.plays_another_song();
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        assert!(
            log.said("restore: the track has changed since this snapshot; taking the music down"),
            "the restore put a buffer back into a stream playing another track:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.music_stops(),
            1,
            "the music was not taken down through the game's own call",
        );
        // And the chapter's own track started again through the game's own call, which answers for both of this
        // game's sound backends — see `Th07::restart_stage_music`. A frame later, the ask being a command the
        // player walks over more than one run of its own sound.
        game.frame();
        assert_eq!(
            game.music_started(),
            vec![TRACK.name.to_str().unwrap().to_owned()],
            "the track that came back is not the one the chapter was taken under",
        );
    });
}

/// And a chapter taken where **nothing is audible** comes back to that silence: the track is asked for, because
/// the player's own stream field is what that ask fills, and the sound it started is taken back off.
///
/// This is the shape a launch of th07 was found in and the one the two cases above do not reach: a stage's
/// first chapter falls frames into the stage — `stage 6 chapter 1 (stage start) at frame 9` — and by then the
/// song of the scene before it has been taken down while the stage's own has not been asked for. So there is a
/// stream to capture and nothing coming out of it, which `music: true` does not tell apart: that line is
/// `Snapshot::has_music`, a stream held rather than a sound heard.
///
/// **The stage's own song must not be left playing**, its script starting it at its own moment: a launch that
/// had orb start it heard it twice, once early and once from its beginning when that moment came — its log
/// saying so outright with `music: restarting slot 0, the chapter's own track being th07_01.wav` and then
/// `restore: the song could not be put`, an offset in the front end's track being no offset in the stage's.
///
/// **And it must be asked for all the same**, which is the other half and the one that cannot be left out: a
/// restore that asked for nothing left the player holding no stream, and the launch that tried it had no music
/// for the rest of the stage — the play command's own block is skipped whole where that field is null — and then
/// faulted at 0x44d023, which reads the stream through it with no test at all.
#[test]
fn a_chapter_of_th07_taken_with_nothing_audible_comes_back_to_that_silence() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-not-yet",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.took_down_the_song_it_came_in_with(HEARD_AT);
        game.in_a_pointdevice_run();
        assert!(
            log.said("music: true"),
            "the chapter was taken with no stream of the music in it, which is another case than this:\n  {}",
            log.lines().join("\n  ")
        );

        // The stage's own song, starting after that chapter was taken — which is the stage's script asking for
        // the slot its build loaded, and a new stream on the same buffer.
        game.its_stage_song_starts();
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // Taken down, which it has to be: the stream playing now was allocated after the chapter, so the memory
        // copy rolls its object out from under the thread streaming into it — see `Snapshot::restore`.
        assert_eq!(
            game.music_stops(),
            1,
            "the music was not taken down through the game's own call",
        );

        // **And the silence waits for the game's own sound to have carried that ask out**, which is more than the
        // one run the restore's own frame gives it: what orb has asked for is a command in the player's own
        // queue, and stopping a buffer the play has not reached yet stops nothing. A launch of th07 heard
        // exactly that — `the track asked for is stopped again` in the log and the song audible after it.
        assert!(
            !log.said("this chapter had nothing audible"),
            "the sound was taken off in the restore's own frame, where the ask it made is still in the queue:\
             \n  {}",
            log.lines().join("\n  ")
        );
        game.frame();

        // And a track asked for, which is what leaves the player holding a stream at all.
        assert_eq!(
            game.music_started(),
            vec![TRACK.name.to_str().unwrap().to_owned()],
            "no track was asked for, so the player is left holding no stream",
        );
        assert_ne!(
            game.image().the_stream_it_names(),
            0,
            "the player was left naming no stream, which its own play command is skipped on",
        );

        // And nothing coming out of it, which is the silence the chapter was taken in.
        assert!(
            log.said(
                "restore: this chapter had nothing audible; the track asked for is stopped again"
            ),
            "the sound the ask started was left playing:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !game.stream_now().playing,
            "the stage's own song was left playing, which its script is about to start itself",
        );
        // And nothing seeked either: a position is a position in a track, and this chapter's sound was in none.
        assert!(
            !log.said("restore: the song"),
            "a position was put into a track for a chapter that had nothing playing:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **The song is put back where the ask for its track has been carried out, and not before**: the play is a
/// command in the player's own queue, that queue is walked a phase at a time, and a command still standing is a
/// track that is not playing yet.
///
/// Its own case because it fails as its own thing, and a launch of th07 is what it fails as: a seek made in the
/// restore's own frame read `restore: the song could not be put`, the record the source was on still being the
/// boss's. So the seek waits for the queue to go empty — see `Game::a_sound_command_is_in_flight` — and what is
/// asserted here is both halves of that: nothing in the restore's own frame, and the chapter's own position once
/// the game's own sound has carried the ask out.
///
/// **And the queue is walked by the frame's own sound rather than from inside the restore**, which is the other
/// half: asking the game to carry a play out while the memory copy has just put the stream it had freed back is
/// asking it to walk that stream. A launch faulted in exactly that — `0x0045d8d5`, reading `4`, with
/// `E_INVALIDARG` in two registers — so what this asks is that a frame's sound is asked for once, the way the
/// game's own frame asks for it.
#[test]
fn the_song_of_th07_is_put_back_where_the_ask_for_its_track_was_carried_out() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-queued",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(HEARD_AT);
        game.in_a_pointdevice_run();
        assert!(log.said("music: true"));

        // The fight's own track, and the file read on for it: the position the stream is at is the boss's
        // music, so a position that is the chapter's again afterwards is one the restore's own seek put there.
        game.plays_another_song();
        game.services_the_buffer();
        assert_ne!(
            game.stream_now().position,
            HEARD_AT,
            "the file was left where the chapter had it, so nothing here says the song was picked up",
        );

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.forget_asked();
        let sound_work = game.image().times_the_sound_commands_ran();
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // One sound a frame, which is what the game's own frame asks for: a second one is orb asking the game
        // to carry a queued play out from inside the restore, over the stream the copy has just put back.
        let sounds = game.asked().iter().filter(|call| **call == SOUND).count();
        let frames = game.asked().iter().filter(|call| **call == PRESENT).count();
        assert_eq!(sounds, frames, "the frame's own sound is not once a frame");
        assert_eq!(
            (game.image().times_the_sound_commands_ran() - sound_work) as usize,
            sounds + 1,
            "the game's sound work ran more often than a frame's own and the stop before the copy, which is orb \
             running the queue over the stream that copy has just put back",
        );

        // Nothing seeked yet, the ask being a command the player has not walked to its end: a seek here is a seek
        // of the track the chapter's own replaced, which is the launch that read `could not be put`.
        assert!(
            !log.said("restore: the song"),
            "the song was seeked in the restore's own frame, where the track asked for is not playing yet:\n  {}",
            log.lines().join("\n  ")
        );
        game.frame();

        assert!(
            log.said(&format!(
                "restore: the song picked up at {HEARD_AT} in the track's own file"
            )),
            "the song was not put where the chapter had it once the ask had been carried out:\n  {}",
            log.lines().join("\n  ")
        );
        // And the file itself is where that seek left it: the chapter's own offset and the buffer filled from
        // there, which is what `Music::play_from` does with one read rather than wait for the streaming
        // thread's first notification.
        assert_eq!(
            game.stream_now().position,
            HEARD_AT + STREAM_BUFFER as i32,
            "the file was left where the track that had replaced the chapter's read to",
        );
    });
}

/// **A stream comes back going round, whatever the flag said when the chapter was taken.**
///
/// A streaming buffer is a ring: the game's own thread writes into it ahead of the mixer and the mixer wraps at
/// its end for as long as the track lasts. So `DSBPLAY_LOOPING` is not a state of the moment a chapter was taken
/// in but what a played stream *is*, and a restore that puts back what the status happened to say stops the
/// music wherever the buffer's end falls.
///
/// A launch of th07 is the measurement, on a resumed run whose fourth stage was retried from its first
/// chapter: `sound: put back at 133329344 in the file (asked 133329344), cursor 0 (asked 0), 42468336 byte(s)
/// left, next write at Some(0), buffer locked, status 0x1 (asked 0), playing`. Every number the restore asked
/// for came back — and `0x1` is playing with the ring bit clear, which is the music stopping partway with the
/// game's own thread still filling a buffer nobody was playing.
#[test]
fn the_stream_of_th07_comes_back_going_round() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-looping",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(HEARD_AT);
        // The buffer playing with the ring bit clear **before the chapter is taken**, which is what that
        // launch's was taken in: what a restore puts back is the state the capture found.
        game.plays_without_looping();
        game.in_a_pointdevice_run();
        assert!(log.said("music: true"));
        assert!(!game.stream_now().looping, "the sound was left going round");

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        let stream = game.stream_now();
        assert!(
            stream.playing && stream.looping,
            "the stream came back {}, so the music stops where the buffer ends:\n  {}",
            if stream.playing {
                "playing once"
            } else {
                "silent"
            },
            log.lines().join("\n  ")
        );
    });
}

/// The buffer the game has made by the time the chapter is put back, which is not the one it was taken with.
///
/// An address and not an object, the same as the texture handles in `the_handles_a_restore_leaves_alone.rs`:
/// orb hands a buffer to DirectSound and reads no word through it, so what this has to be is a number that can
/// be told from the one the chapter had.
const BUFFER_MADE_SINCE: usize = 0x0700_0000;

/// **The buffer the game is playing through is not put back from a snapshot.**
///
/// DirectSound made it and nothing in the game's own memory holds it but the one word at `[[stream+0x4]]` — so
/// a restore that writes that word back from a chapter names whatever object was there *then*, and the game
/// released that one when it made the buffer it is playing through now. What orb reads through it is refused,
/// and so is every call the game's own streaming thread makes: the buffer that is actually playing is never fed
/// again, so it goes round its own contents and the game's own next track never reaches it.
///
/// **Which is what a th07 session was heard as** — a few seconds of stage 5's boss repeating after a chapter
/// retry, and the same few seconds still playing at the title — and what its log said, one frame after the
/// restore and every reading after it: `status 0x0 (asked -2147024809)`, `E_INVALIDARG` off `GetStatus`, with
/// the countdown and the next write frozen at the values that chapter had.
#[test]
fn the_buffer_a_stream_of_th07_names_is_not_put_back_from_a_snapshot() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-buffer-a-restore-leaves",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(HEARD_AT);
        game.in_a_pointdevice_run();

        // A chapter of the fight the stage ends with, whose sound a restore leaves alone — the arm where
        // nothing else about the stream is put back either.
        game.frames(A_STRETCH);
        game.the_stages_own_boss_arrives();
        game.frames_until("the fight's own chapter", A_STRETCH, || {
            log.lines()
                .iter()
                .any(|line| line.contains("chapter 2 at frame") && line.contains("music=keep"))
        });

        // What the chapter was taken with, and then the buffer the game has made since: a track change deletes
        // the one that was playing and makes another somewhere else, so by the time this chapter comes back the
        // word holds an object the chapter knew nothing about — and the one it did know has been released.
        assert_eq!(
            game.image().the_buffer_its_stream_names(),
            orb_sim::Sound::the_buffer(),
            "the chapter was not taken with the buffer this test thinks it was",
        );
        game.image().its_stream_names_the_buffer(BUFFER_MADE_SINCE);

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 2"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        assert_eq!(
            game.image().the_buffer_its_stream_names(),
            BUFFER_MADE_SINCE,
            "the restore put back a buffer the game had released:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// How long a stretch of a fight is here, which is only the frames a chapter of one needs.
const A_STRETCH: u32 = 60;

// **A chapter left playing has its stream left alone, and there was a test here that asked the opposite.**
//
// It laid the game's own next write inside the chunk the device is writing out of — read off th07's 0x45db60 as
// the geometry that refuses a refill — and asked that a restore rebuild the stream there. The state it laid out
// is the *ordinary* one: measured over a run of retries on the real game, the next write sits exactly one chunk
// behind that cursor at every reading and the thread goes on feeding the stream. So the test passed while the
// repair it was about rebuilt the stream on every retry, each from an offset seconds further on than the last.
//
// **What it could not have caught is the half a laid-out game does not have**: no streaming thread, so nothing
// here can say whether the game's own goes on feeding a stream. That is `audio::read_the_stream_back`'s, on the
// real game, and it is what said this.

/// **And the music of a run whose chapter was asked for from the game's own pause menu carries on**, that
/// menu's 「最初からやり直す」 being the one way out of it that sends no un-pause of its own.
///
/// The manager pauses the music on the frame the menu goes up (0x42d9b1), and every way out of it that leaves
/// the run standing sends the un-pause back (0x403496). This item sends none: what it asks for is the scene
/// that gives the run up and begins it again, which would build the run and its music from nothing. orb answers
/// that ask with a question of its own and leaves the scene where it is — so the un-pause is orb's, and a th07
/// session is what its absence cost: from a chapter retried at stage 5's boss to the end of the run, silence,
/// with the song back only after the title and つづきから.
#[test]
fn the_music_of_th07_carries_on_where_the_pause_menu_asked_the_run_to_start_over() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-music-asked-to-start-over",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(HEARD_AT);
        game.in_a_pointdevice_run();
        assert!(
            game.stream_now().playing,
            "the song was not playing to start with",
        );

        // The menu button, the way somebody playing opens that menu — and the music paused with it, which is
        // the state its own items are chosen in.
        game.press(keys::ESCAPE);
        assert!(
            game.state().paused,
            "the menu button did not put the menu up",
        );
        assert!(
            !game.stream_now().playing,
            "the game's own menu did not pause the music, so there is nothing here to carry on",
        );

        // 最初からやり直す: the menu down and the scene asked for, with nothing sent to the music.
        game.image().the_pause_menu_asks_to_start_over();
        game.one_frame();
        assert!(
            log.said("retry: the run was asked to start over in chapter 1"),
            "the ask was not noticed:\n  {}",
            log.lines().join("\n  ")
        );

        // And the music playing again where orb's question stands: the run carries on where it was, and its own
        // music with it.
        assert!(
            game.stream_now().playing,
            "the rest of the run is played out in silence:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("retry: the music un-paused"),
            "orb did not say it had un-paused the music:\n  {}",
            log.lines().join("\n  ")
        );

        // **And the stream built again rather than only started**, which is what the mixer having run on while
        // nothing fed it needs: the buffer filled from where the song is audible, and the game's own next write
        // at that buffer's start — the one relation its refill carries on from.
        assert!(
            log.said("sound: the song built again from"),
            "the stream was left as the un-pause found it:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().next_write_offset(),
            0,
            "the game's own next write is not where a filled buffer puts it",
        );
        assert_eq!(
            game.stream_now().position,
            HEARD_AT + STREAM_BUFFER as i32,
            "the file is not a buffer's worth past where the song is audible",
        );
    });
}
