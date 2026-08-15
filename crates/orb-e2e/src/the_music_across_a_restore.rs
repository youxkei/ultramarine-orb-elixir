//! **The music put back where a chapter had it, and the stream believing the file it was given.**
//!
//! Two of the things this is about can only be told by ear, which is what the numbers beside them stand in
//! for here.
//!
//! What the laid-out game brings is a stage that streams the two songs its data names —
//! `Fake::plays_its_songs`, the stage's own and the boss's — which is the whole of what the question
//! *which* chapters put their music back turns on. And where an e2e test is about the stream itself
//! rather than about which chapters rewind, `Fake::streams_its_song` gives it a real buffer and a real
//! pair of winmm functions to call, which is what the seek, the byte-for-byte restore and the margin are
//! read off.

use crate::fake::th06::{
    CARD_STARTS, Fake, INVULNERABLE_AFTER_SPAWNING, STAGE_BOSS_ARRIVES, STREAM_BUFFER,
    STREAM_NOTIFY, the_run,
};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th06::Th06;
use orb_sim::keys;

/// What a chapter's line says about its music: put back with the chapter, or left playing through it.
const REWIND: &str = "music=rewind";
const KEEP: &str = "music=keep";

/// Which chapter the midboss's arrival is, counting the stage's own start as the first: the fight is the
/// second, and the card it puts up and the attack after that are the two beside it.
const BOSS_ARRIVES_AT_CHAPTER: u32 = 2;
const MIDBOSS_CHAPTERS: [u32; 3] = [2, 3, 4];

/// A pointdevice run over a stage that streams its songs, played as far as `frames`.
fn playing(name: &str, frames: u32) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.plays_its_songs();
    game.in_a_pointdevice_run();
    game.frames_until("the stage played through", frames * 2, || {
        game.state().stage_frames > frames
    });
    game
}

/// What a chapter's line said about its music, for the chapter of that number.
///
/// # Panics
/// Where no such chapter was taken, with every chapter line: an e2e test reading the music of a chapter
/// that never began is about to assert on a line that is not there.
fn music_of(game: &Fake, chapter: u32) -> String {
    let lines = game.log().lines();
    let looking_for = format!("chapter {chapter} at frame");
    lines
        .iter()
        .find(|line| line.contains(&looking_for) && line.contains("music="))
        .unwrap_or_else(|| {
            panic!(
                "no chapter {chapter} was taken:\n  {}",
                lines
                    .iter()
                    .filter(|line| line.contains("music="))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n  ")
            )
        })
        .clone()
}

/// A chapter's song position comes back with the chapter, and it is asked of the song rather than the kind.
///
/// Asked of the chapter's *kind*, a run picked up at a midboss came back with the song at the track's
/// opening and the position that had been written down never read: a midboss is not one of the midstage
/// kinds, although the stage's own song plays through it. It is asked of the song now, from the same place a
/// retry asks it, so the two cannot disagree about a chapter.
///
/// **What is asserted here is the question, not the seek**: which chapters have their song put back is what
/// the kind and the song disagreed about, and a midboss chapter answering *rewind* is that disagreement
/// settled. The seek itself is `a_sought_stream_keeps_its_countdown_and_still_takes_its_loop`'s.
#[test]
fn a_chapters_song_position_is_asked_of_the_song_and_not_of_the_chapters_kind() {
    in_its_own_process(|| {
        let game = playing("the-music-the-midboss", CARD_STARTS + 60);

        // The midboss's own chapters, which are the ones the two answers differ on: a midboss is a fight,
        // so its chapters are not of any midstage kind — and the song playing through it is the stage's, so
        // asked of the song they are chapters whose music is put back.
        for chapter in MIDBOSS_CHAPTERS.into_iter().take(2) {
            let line = music_of(&game, chapter);
            assert!(
                line.contains(REWIND),
                "a midboss chapter was left holding the song wherever the run had got to: {line}",
            );
            assert!(
                line.contains(", boss"),
                "chapter {chapter} is not one of the midboss's, so it says nothing about the kind: \
                 {line}",
            );
        }
    });
}

/// Seeking the file moves the countdown with it, or the stream reads past the end and takes no loop.
///
/// Heard once near the end of a resumed stage 1's midstage and once near the end of a resumed stage 2's: a
/// section of the track repeating. Read off the game afterwards — `WaveFile::Read` (**0x43c080**) clamps
/// every read to `m_ck.cksize` (**0x43c1aa**) and subtracts what it read from it (**0x43c1be**), and
/// `StreamingSound::ServiceBuffer` takes the track's loop only where a read comes up *short against that
/// countdown* (**0x43b759** → `ResetFile(TRUE)` at **0x43b76f**). The file's own end is never asked.
///
/// So seeking the file **5,900,628** bytes forward and leaving the countdown alone left the stream
/// believing it had that much more sound than the file held: it read past the end of the `data` chunk,
/// where `Read` fails rather than returning short — `mmioAdvance` leaves nothing to copy (**0x43c233**) —
/// so no loop was taken and the buffer went round its own contents. It came right by itself once the
/// skipped bytes had been counted off, which is why it was one episode and not a hang.
///
/// The countdown is moved with the file now, to the loop point less where the file ends up, and the loop
/// point is taken as the position plus the countdown as they stand — the pair the game reads, rather than
/// the header's loop fields, since a track without a loop end runs on `cksize` instead. **Not re-heard**:
/// what a laid-out game can say is that the pair still names the same loop point afterwards, which is the
/// arithmetic the episode was the failure of.
#[test]
fn a_sought_stream_keeps_its_countdown_and_still_takes_its_loop() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-the-seek", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // The sound the track is streamed through, with what is audible well inside the file: a seek back to
        // the opening milliseconds would move the countdown by so little that leaving it alone would say
        // nothing.
        game.streams_its_song(SONG);
        let loops_at = game.loop_point();
        game.in_a_pointdevice_run();

        // A chapter of the midboss's, which is what the run is picked up into: the position it was written
        // down with is the one the resume seeks to.
        game.frames_until("the chapter the card is", 900, || {
            game.log().said(&format!("at frame {CARD_STARTS}"))
        });
        game.one_frame_to_drain_the_log();
        assert!(
            game.log().said("(MIDBOSS SPELL 1) at frame"),
            "the chapter the run is picked up into was not written down:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The run given up and started again, and つづきから: the stage is built, the buttons the run
        // pressed are played back into it, and the song is put where that chapter had it.
        gives_the_run_up(&game);
        game.picks_the_run_up();
        assert!(
            game.log()
                .said(&format!("resume: the song picked up at {SONG}")),
            "the song was not put back where the chapter had it:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the countdown went with the file: the pair still names the loop point it named before, which
        // is what says the stream will take its loop where the track does rather than read past the end of
        // its own sound.
        assert_eq!(
            game.loop_point(),
            loops_at,
            "the seek moved where the track loops, which is a stream that reads past the end of the \
             sound and takes no loop at all",
        );
        // Said out loud, since the arithmetic above holds trivially for a seek that never happened: the
        // countdown is smaller than it was, by the sound that was skipped and the buffer that was filled.
        assert!(
            game.image().bytes_left() < loops_at - SONG as u32,
            "the countdown was left where it was, which is the stream believing it has more sound \
             left than the file holds",
        );
        game.one_frame_to_drain_the_log();
        assert!(
            game.log()
                .said(&format!("music: the track loops at {loops_at}, so")),
            "orb did not say what it worked the countdown out to be:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// How far into the track the sound now audible begins, for the e2e test above.
///
/// Well inside the file and past a buffer's worth of it, which is what the episode needed to happen: the
/// real one was **5,900,628** bytes into `th06_02.wav`.
const SONG: i32 = 300_000;

/// A chapter's sound comes back with the chapter: the bytes in the buffer, the play cursor in them, and
/// the file the next chunk is read out of.
///
/// **Three pieces of the music live outside the game's memory**, so a snapshot of that memory holds none
/// of them: the buffer belongs to DirectSound, the cursor to its mixer, and the position to winmm's
/// `HMMIO`. Putting the memory back without them leaves the streaming bookkeeping describing a buffer
/// that has moved on, which is audible as a short loop repeating forever — the fault `audio.rs` exists
/// for.
///
/// Which chapters that happens to is
/// `the_track_is_rewound_for_the_chapters_that_share_it_and_left_alone_for_a_boss`'s. This is the *what*:
/// the four values read either side of a retry, with the stream moved on in between so that coming back is
/// something that had to happen rather than something nothing disturbed.
#[test]
fn a_chapters_stream_comes_back_byte_for_byte_when_the_chapter_does() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-put-back", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.streams_its_song(SONG);
        game.in_a_pointdevice_run();

        // A chapter of the midboss's, which the stage's own song plays through — so one whose music is
        // put back rather than left playing. The stream is where the chapter's snapshot found it: nothing
        // here moves it but an e2e test saying so.
        game.frames_until("the chapter the card is", 900, || {
            game.log().said(&format!("at frame {CARD_STARTS}"))
        });
        let at_the_chapter = game.stream_now();
        let bookkeeping = (game.image().next_write_offset(), game.image().bytes_left());
        assert!(
            game.log().said("music=rewind"),
            "the chapter was taken with nothing of its music in it:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And then the streaming thread runs: two chunks read out of the file into the buffer, and the
        // mixer part way through what was already in there. Every one of the four moves.
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
        game.hit();
        game.frame();
        let log = game.log();
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // The stream is the one the chapter was taken with, byte for byte — and it is still the same
        // stream through the same buffer, which is what says the sound was put back rather than taken
        // down and started again.
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
        // And the game's own bookkeeping came back with the rest of its memory, which is the half of the
        // pair that has to agree with the buffer: the offset the next chunk goes at, and the countdown
        // the track's loop is taken on.
        assert_eq!(
            (game.image().next_write_offset(), game.image().bytes_left()),
            bookkeeping,
            "the streaming fields describe a buffer that is no longer there",
        );
    });
}

/// The distance between the play cursor and the offset the next chunk goes at is watched, and a chapter
/// restored on a timer leaves it where the chapter had it.
///
/// **What a listener hears when that distance runs out is the music breaking up**, which is why orb
/// reports it at all — and it only asks DirectSound for it while something is being diagnosed, a poll a
/// frame being work nothing else needs. `--stress-restore` is one of the two switches that turn it on,
/// and it is also what puts a restore on a timer: the snapshot and the music get as many goes as a long
/// session would with nobody playing one.
#[test]
fn the_distance_to_the_next_write_is_reported_and_a_restore_on_a_timer_puts_it_back() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-watched", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.stress_restore_frames = RESTORED_EVERY;
        });
        game.streams_its_song(SONG);
        // The mixer this far past the offset the next chunk goes at, which is what the stream is when it
        // is keeping up: inside a chunk of it. Said before the run starts, so it is the distance every
        // chapter is taken with and so the one every restore has to put back.
        game.plays_the_buffer_on(BEHIND);
        game.in_a_pointdevice_run();

        // The restore on its timer, over and over: four goes at a chapter and then the run walks on.
        game.frames_until(
            "the chapter restored on a timer",
            RESTORED_EVERY * 8,
            || game.log().said("retry chapter 1 (retry 4)"),
        );
        // And what the frames after each of those restores said the distance was, which is the number the
        // restore had to bring back: the cursor is put back through the buffer and the offset comes back
        // with the game's memory, so a restore that moved either of them would read as another number.
        assert!(
            game.log().said(&format!(
                "audio: behind={BEHIND} of chunk {STREAM_NOTIFY} buffer {STREAM_BUFFER}"
            )),
            "the distance after a restore was not the one the chapter was taken with:\n  {}",
            game.log()
                .lines()
                .iter()
                .filter(|line| line.contains("audio:"))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        // Reported as a range once a reporting period, which is where a run's worst is read off.
        game.frames_until_the_log_holds_another("audio: behind ");
        assert!(
            game.log()
                .said(&format!("audio: behind {BEHIND}..{BEHIND} bytes")),
            "the period's report is not the distance every frame of it measured:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Where the track has changed since the snapshot, the sound is taken down and started again through the
/// game rather than copied back.
///
/// **A released COM object does not come back from a memory copy.** Once the game has changed track it has
/// freed the stream and released its sound buffer, and neither is the memory restorable around the stream
/// that replaced it: that one was allocated after the snapshot, so writing the snapshot back rolls its
/// object out from under a streaming thread that is not suspended — measured as an access violation inside
/// `DSOUND.dll` writing a buffer it no longer owned.
///
/// So the sound goes down through the game's own `StopBGM` before the copy — its allocator has to see the
/// stream being freed — and comes back through its own `PlayAudio` after it, given the path read out of
/// the memory that was just put back.
///
/// Reached here the way a run reaches it: ステージをやり直す from inside the fight the stage ends with,
/// whose own track is not the one the stage's start was snapshotted with.
#[test]
fn a_chapter_whose_track_has_gone_is_restored_by_taking_the_music_down_and_starting_it_again() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-restarted", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // A stage that streams its song through a real buffer *and* names the two songs its data names, so
        // that the fight the stage ends with brings the second of them — which is the track change this is
        // about.
        game.streams_its_song(SONG);
        game.plays_its_songs();
        game.image().names_its_song(STAGE_SONG);
        game.in_a_pointdevice_run();
        assert_eq!(
            game.music_stops(),
            0,
            "the music was taken down before anything asked for it",
        );

        // Into the fight the stage ends with, which is where the boss's own track starts: from there the
        // stage's start is a chapter whose stream has been freed.
        game.frames_until(
            "the fight the stage ends with",
            STAGE_BOSS_ARRIVES * 2,
            || game.state().stage_frames > STAGE_BOSS_ARRIVES,
        );

        // 被弾, and then ステージをやり直す — the retry menu's third item, two presses down from the first:
        // this stage has chapters behind the one that was lost, so the item that lists those is between
        // them.
        let log = game.log();
        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the stage asking", || {
            log.said("retry: asking about the stage again")
        });
        // And the question it asks, whose cursor starts on いいえ — the answer that costs nothing.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the stage asked for again", || {
            log.said("retry: the stage again on the keyboard")
        });

        // The track was recognised as not the one the snapshot holds, and the sound went down through the
        // game rather than being copied back over a buffer DirectSound has freed.
        assert!(
            log.said("restore: the track has changed since this snapshot; taking the music down"),
            "the restore tried to put back a stream that had been freed:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("music: stopped through the game"),
            "the sound was not taken down through the game's own StopBGM:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.music_stops(),
            1,
            "the game's own StopBGM was not the call that took the music down",
        );

        // And came back through the game's own PlayAudio, by the path the restored memory names — which is
        // the stage's own song and not whatever was playing when the chapter was left.
        assert!(
            log.said(&format!("music: restarting {STAGE_SONG}")),
            "orb did not read the stage's song out of the memory it had just put back:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.music_started(),
            vec![STAGE_SONG.to_owned()],
            "the track was started again by some other path than the one the stage names",
        );

        // And it was put where that chapter had the song rather than left at the track's opening: the
        // stream that held the place has been freed, so what is left of it is the offset in the track's
        // own file — which is what a resume puts a landing's song back with, for the same reason.
        assert!(
            log.said(&format!("restore: the song picked up at {SONG}")),
            "the track was started again at its beginning:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.stream_now().position - SONG,
            STREAM_BUFFER as i32,
            "the file is not one buffer past the sound that is audible",
        );

        // ── And the chapter holds that stream from now on: 被弾 again where the restore left the run,
        // and the sound comes back the ordinary way — the buffer and the cursor put back over a stream
        // that is still there. Taken again after the restart, it would otherwise name the stream the
        // game freed for ever, and every death in this chapter would take the track down again.
        //
        // Past the frames a stage starts its player invulnerable for, and still inside the chapter the
        // restore put back: the next boundary of this stage is the fight at 900.
        game.frames_until("the player killable again", 300, || {
            game.state().stage_frames > INVULNERABLE_AFTER_SPAWNING as u32
        });
        let from = log.written();
        game.hit();
        game.frame();
        game.press_until(keys::Z, "the chapter again", || {
            log.said_since(from, "retry: the chapter again on the keyboard")
        });
        assert!(
            !log.said_since(from, "taking the music down"),
            "the track was taken down a second time for a chapter whose stream is playing:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.music_stops(),
            1,
            "the game's own StopBGM was called again for a chapter that could be rewound",
        );
    });
}

/// The path the stage's data names its own track by, as 紅魔郷 writes one.
const STAGE_SONG: &str = "bgm/th06_02.mid";

/// A stream whose file handle orb cannot read has the buffer and the cursor put back and the file left
/// where it is.
///
/// **Which is a stream that happens rather than one laid out to be awkward.** `Th06::music` takes the
/// handle as nothing where the read of it does not come off — the whole chase to it is pointers out of
/// structures the game rebuilds between stages — and both winmm calls refuse a handle of nothing. So what
/// a chapter written down then holds is a file position of `-1`, which is `mmioSeek` saying it will not
/// say, and a restore leaves the file alone rather than seeking somewhere wrong.
///
/// The rest still comes back, which is the point: the two pieces orb reaches by *calling* DirectSound are
/// independent of the one it reaches through winmm, and a run that loses the third is a run whose music is
/// a little out of place rather than one whose music breaks up.
#[test]
fn a_stream_whose_file_handle_will_not_read_keeps_its_buffer_and_leaves_the_file_alone() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-no-handle", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.streams_its_song(SONG);
        game.image().forgets_the_file_handle();
        game.in_a_pointdevice_run();

        game.frames_until("the chapter the card is", 900, || {
            game.log().said(&format!("at frame {CARD_STARTS}"))
        });
        let at_the_chapter = game.stream_now();

        // The streaming thread runs. Its own reads do not go through the game's handle — an e2e test says
        // the thread ran, and what it reads is the file itself — so the position moves even though orb
        // cannot ask where it is.
        game.services_the_buffer();
        game.plays_the_buffer_on(STREAM_NOTIFY);
        let moved_on = game.stream_now();
        assert_ne!(moved_on, at_the_chapter, "the stream did not move");

        // 被弾 and チャプターをやり直す.
        let log = game.log();
        game.hit();
        game.frame();
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // The buffer and the cursor are the chapter's, and the file is where the thread left it.
        let back = game.stream_now();
        assert_eq!(
            (back.buffered, back.play_cursor, back.playing),
            (
                at_the_chapter.buffered,
                at_the_chapter.play_cursor,
                at_the_chapter.playing
            ),
            "the buffer and the cursor did not come back with the chapter",
        );
        assert_eq!(
            back.position, moved_on.position,
            "the file was seeked with a handle that answers nothing about where it is",
        );
    });
}

/// How often that e2e test's restore comes round, in frames. Its own number: short enough that a chapter
/// gets its four goes inside an e2e test and long enough that the frames between them are frames.
const RESTORED_EVERY: u32 = 20;

/// And how far the play cursor is ahead of the offset the next chunk goes at. Inside a chunk, which is a
/// stream that is keeping up — the number growing past one is the streaming falling behind.
const BEHIND: u32 = 3_000;

/// 被弾 and then タイトルに戻る, which leaves the chapter behind for the next launch to offer.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.hit();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
}

/// The track is rewound for the chapters that share the stage's theme and left playing through a boss's.
///
/// Rewound for the midstage and the midboss, which share the stage theme, and left playing through a
/// boss-fight restore.
///
/// And where the track has changed since the snapshot, it is asked to start again rather than copied back:
/// `StopBGM` and then `PlayAudio`, given the path read out of the memory the restore had just put back.
/// **That half is the real game's**, taking the music down and starting it again being calls into 紅魔郷's own
/// two functions at their own addresses, and there is no code at those addresses in a game laid out by hand.
#[test]
fn the_track_is_rewound_for_the_chapters_that_share_it_and_left_alone_for_a_boss() {
    in_its_own_process(|| {
        let game = playing("the-music-which-chapters", STAGE_BOSS_ARRIVES + 60);

        // The stage's own theme plays through the midstage and the midboss alike, so every chapter up to
        // the fight the stage ends with is one whose music is put back with it. The midboss's are what say
        // so here: the stage's own first chapter is announced by a line of its own, which says whether the
        // music had come up rather than what will be done with it.
        for chapter in MIDBOSS_CHAPTERS {
            let line = music_of(&game, chapter);
            assert!(
                line.contains(REWIND),
                "a midboss chapter, which the stage's own theme plays through, was not rewound: {line}",
            );
        }

        // And the fight the stage ends with brings the second of the two songs its data names, which is
        // what says this fight is the stage's own rather than another midboss: from there the music is left
        // playing, because a boss's own track is not the stage's to rewind.
        let boss = game
            .log()
            .lines()
            .iter()
            .filter(|line| line.contains("music="))
            .filter(|line| {
                line.split("at frame ")
                    .nth(1)
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|frame| frame.parse::<u32>().ok())
                    .is_some_and(|frame| frame >= STAGE_BOSS_ARRIVES)
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            !boss.is_empty(),
            "no chapter began at the fight the stage ends with, so nothing here says which song it \
             was fought to:\n  {}",
            game.log().lines().join("\n  ")
        );
        for line in &boss {
            assert!(
                line.contains(KEEP),
                "the boss's own track was rewound with a chapter of its fight: {line}",
            );
        }

        // Which is not the answer the chapter's *kind* gives: the midboss's chapters and the stage boss's
        // are fights alike, and the two came out differently — so what decided them was the song.
        // Which is not the answer the chapter's *kind* gives: the midboss's chapters and the stage boss's
        // are fights alike — every one of them says `, boss` — and the two came out differently, so what
        // decided them was the song.
        assert!(
            music_of(&game, BOSS_ARRIVES_AT_CHAPTER).contains(", boss")
                && boss[0].contains(", boss"),
            "the two fights are not both fights, so nothing here shows the kind was not the test",
        );
    });
}

/// **A stream whose buffer is no longer a live object is a stage whose chapters hold no sound.**
///
/// The word at the buffer's head is the whole of what orb has to tell a live COM object from the stale
/// pointer a released one left behind, and it looks at it because the alternative is calling through that
/// pointer: the game releases the buffer the moment it changes track, and a snapshot that copied the object
/// back would be putting a freed one under the streaming thread.
///
/// So `Th06::music` answers nothing, and the whole of what follows is that: the stage waits out the whole of
/// the wait for a track that will never be readable, takes its first chapter without sound, and plays on.
/// Which is worth holding orb to because the run is still a run — the chapters rewind, the retry menu works,
/// and only the music is gone.
#[test]
fn a_stream_whose_buffer_has_been_freed_leaves_the_chapters_without_sound() {
    in_its_own_process(|| {
        let game = Fake::attach("the-music-freed", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // A real buffer first, so that what the next line does is take a live one away rather than never
        // lay one out: a stage with no stream at all is a different launch.
        game.streams_its_song(0);
        game.plays_its_songs();
        assert!(
            Game::music(&Th06).is_some(),
            "the stream was not readable before this e2e test freed its buffer",
        );
        game.image().frees_the_stream_buffer();
        assert!(
            Game::music(&Th06).is_none(),
            "orb still reads a stream whose buffer is not a live object",
        );

        // The run goes as it always does, and its first chapter is taken with no sound in it.
        game.in_a_pointdevice_run();
        assert_eq!(
            game.log()
                .lines()
                .iter()
                .filter(|line| line.contains("chapter 1 (stage start)")
                    && line.contains("music: false"))
                .count(),
            1,
            "the stage's first chapter reports sound it cannot have read:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the run is still a run: 被弾, チャプターをやり直す, and the chapter comes back.
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            game.log().said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });
        let at_the_card = game.state();
        game.hit();
        game.frame();
        assert!(game.log().said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            game.log().said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "a chapter with no sound in it did not come back",
        );
    });
}
