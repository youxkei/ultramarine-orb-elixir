//! **The music of 妖々夢 put back where a chapter had it, out of the file the game opens itself.**
//!
//! What is being asked is the same thing `the_music_across_a_restore.rs` asks of 紅魔郷 — the buffer's bytes,
//! the mixer's cursor and how far the file has been read, none of which is memory and so none of which a
//! snapshot holds — through the half of it that is *not* the same: 紅魔郷 opens a wav per track through
//! winmm's `mmio`, and 妖々夢 opens `thbgm.dat` once itself and reads every track out of that handle. So a
//! position here is an absolute offset in the archive, moved with `SetFilePointer` and read with `ReadFile`,
//! which is `orb_core::audio::Track`'s second shape.
//!
//! The stream is a real buffer and those two are real functions, because orb reaches them by calling rather
//! than by reading — see `orb_sim::Sound`. What no e2e test can say is what any of it sounds like; the numbers
//! below stand in for that.

use crate::fake::th07::{Fake, STREAM_BUFFER, STREAM_NOTIFY, TRACK};
use crate::fake::{Display, Launched, PRESENT, SOUND, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How far into the track the sound now audible begins when the chapter is taken: part way in, so that a
/// position put back to the beginning of the file is a different number from the one that was there.
const HEARD_AT: i32 = 0x100;

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

        // 被弾 and チャプターをやり直す.
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
/// first record, the title screen's song. That is what a 妖々夢 session heard where the chapter's own track was
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
/// This is the shape a launch of 妖々夢 was found in and the one the two cases above do not reach: a stage's
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

        // **And the silence taken off in the restore's own frame**, the ask having been carried out there: what
        // a restore asks for is the call that *makes* a stream — `Th07`'s `PLAY_BGM` — so the buffer being
        // stopped is one that exists and is playing. While the ask was a queued command instead, stopping it
        // here stopped nothing, and a launch of 妖々夢 heard exactly that: `the track asked for is stopped
        // again` in the log and the song audible after it.
        assert!(
            log.said("this chapter had nothing audible"),
            "the sound this chapter had none of was not taken off in the restore's own frame:\n  {}",
            log.lines().join("\n  ")
        );

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
/// Its own case because it fails as its own thing, and a launch of 妖々夢 is what it fails as: a seek made in the
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

        // **And the song put where the chapter had it in the restore's own frame**, the track being playing by
        // then: what a restore asks for is the call that *makes* a stream — `Th07`'s `PLAY_BGM` — which is
        // carried out where it is made and leaves nothing in the player's queue. A seek made while the ask was
        // still a queued command was a seek of the track the chapter's own replaced, and the launch that read
        // `could not be put` is what said so; the ask is no longer queued, so there is nothing to wait for.
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
