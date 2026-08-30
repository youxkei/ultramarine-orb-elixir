//! **What the music coming out is, in the log, for a game that does not name its tracks.**
//!
//! th06 builds the `.wav` and `.pos` paths of a track in stack buffers at `Supervisor::PlayAudio` and hands
//! them to `SoundPlayer::LoadBGM`, which opens the file through a stream of its own and keeps the mmio handle,
//! the length and the loop points of what it opened — so there is no name in its memory to read. See
//! `docs/th06/asm.md`.
//!
//! What its line carries instead is [`orb_core::game::Game::music_identity`], the number a track's own
//! numbers make: it names nothing and it tells one track from another, which is what a check on the music
//! needs. `the_track_heard_of_th07.rs` is the half where the game does keep the name.

use crate::fake::th06::{CARD_STARTS, Fake, STAGE_BOSS_ARRIVES, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// How far into the track its position is put, which is what makes a chapter's own place in it something
/// other than the beginning — the same number `the_music_across_a_restore.rs` uses.
const SONG: i32 = 300_000;

/// The name the stage's own data holds for its song, as that file's records carry it.
const STAGE_SONG: &str = "bgm/th06_02.mid";

/// The frames a track is left playing before the next one: enough that a line said once a frame would show
/// as many.
const APART: u32 = 60;

/// **And the stop said from inside the game's own stop of the music.**
///
/// Which is the other half of where a sound is made, and the half no reading from the frame can date: a buffer
/// stopped between two frames is stopped by the time anything asks, with nothing saying when or at whose ask.
///
/// **The ask this game's own stop is reached by is a restore under another track**: a chapter taken while the
/// stage's own song played, the fight's track started over it, and that chapter asked for again — the stream
/// the snapshot holds was freed with the track it belonged to, so the sound goes down through
/// `SoundPlayer::StopBGM` before the memory is put back. Which is orb's own ask, and it comes through the same
/// function the game's asks do — see `Hooks::stops_the_music`.
#[test]
fn the_stop_of_th06s_music_is_said_from_inside_the_games_own_stop() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-stop", the_run(), |config| {
            config.log_level = LogLevel::Normal;
        });
        let log = game.log();
        // A stage that streams its song through a real buffer and names the two songs its data names, so that
        // the fight the stage ends with brings the second of them — which is the track change this needs.
        game.streams_its_song(SONG);
        game.plays_its_songs();
        game.image().names_its_song(STAGE_SONG);
        game.in_a_pointdevice_run();
        game.frames_until(
            "the fight the stage ends with",
            STAGE_BOSS_ARRIVES * 2,
            || game.state().stage_frames > STAGE_BOSS_ARRIVES,
        );

        // The death, and ステージをやり直す — the retry menu's third item, two presses down from the first.
        let from = log.written();
        game.dies();
        game.frames_until("the death seen", 8, || {
            log.said_since(from, "died in chapter")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the stage asking", || {
            log.said_since(from, "retry: asking about the stage again")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the stage asked for again", || {
            log.said_since(from, "retry: the stage again on the keyboard")
        });

        assert!(
            log.said_since(from, "sound: stopped through the game"),
            "nothing was said from inside the game's own stop, so a silence has no line to be dated \
             by:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A chapter whose sound was nothing comes back to that nothing when the run is picked up.**
///
/// The position of a stopped stream is where its sound *would* start, so putting it back and playing it makes
/// a silence audible — and a stage's own first chapter is taken in exactly that: it falls frames into the
/// stage, the song of the scene before it has been taken down, and the stage's own has not been asked for. A
/// launch of th07 picked stage 6's first chapter up and played the title screen's own theme for the five
/// seconds until that stage's script started its song.
///
/// **The restore has held a chapter to this all along** — `restore: this chapter had nothing audible; the
/// track asked for is stopped again`, and `the_music_across_a_restore.rs` is where this game's own test of it
/// is — and the resume owes the same.
#[test]
fn a_resume_of_th06_leaves_the_song_alone_where_the_chapter_had_no_sound() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-song-of-a-silent-chapter", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        // The stream there with its mixer stopped, which is the state a stage's first chapter is taken in.
        game.took_down_the_song_it_came_in_with(SONG);
        game.in_a_pointdevice_run();
        game.frames_until("the chapter the card is", 900, || {
            log.said(&format!("at frame {CARD_STARTS}"))
        });
        game.one_frame_to_drain_the_log();

        // The hit and then タイトルに戻る, which leaves the chapter behind for the pick-up to offer.
        game.dies();
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
        let from = log.written();
        game.picks_the_run_up();

        assert!(
            !log.said_since(from, "resume: the song picked up at"),
            "the position of a chapter that had no sound was put into the stream and played:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            log.said_since(
                from,
                "nothing was coming out of it when the chapter was taken"
            ),
            "the landing said nothing about the silence it was leaving alone:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A file from before a chapter said anything about its song has that song put back all the same.**
///
/// Which is what a run of th06 on somebody's disk has had since orb shipped: its midstage music is picked up
/// where the chapter left it. The two fields this build writes — which track the position was read from, and
/// whether any of it was coming out — are what the guards above read, and a file from before them says
/// neither. Refusing what it cannot vouch for would take the music away from every one of those runs, so what
/// a file cannot say orb does not claim to know better than the build that wrote it.
///
/// **The keys taken out of the file the run wrote**, which is how the bytes a released orb wrote are reached
/// from here: the run is played and left as any other, and then its chapter loses the two fields before the
/// pick-up reads it.
///
/// **And what is asserted is where the track ends up, not that a line was said.** A song put back at the
/// track's opening is a song the log calls put back and a listener calls started over, which is the whole of
/// what a run of th06 on a disk stands to lose: the chapter is [`SONG`] bytes into a track whose loop is
/// millions of bytes long, so the file's own position, the countdown that goes with it, and the buffer
/// coming out are the three things that say the music carried on from where the chapter left it.
#[test]
fn a_th06_file_from_before_the_songs_track_has_its_song_put_back() {
    picked_up_out_of(
        "th06-the-song-of-an-older-file",
        without_the_songs_fields,
        "a file from before those two fields",
    );
}

/// **And so does the file the release wrote**, which is the run of anybody who downloaded the exe the README
/// points at.
///
/// The shape is v0.10.0's own, read off `crates/orb-core/src/resume.rs` at that tag: version 1, no cherry
/// among the numbers a stage starts with, no generators, and a chapter that ends at its song position. Three
/// things went in after that release, and any one of them being *required* is that release's runs refused —
/// which is what they were, until `SHIPPED` went in beside `VERSION`.
///
/// **A separate case from the one above and not a stricter version of it**: that file is what the build
/// before this one wrote, this one is what the release wrote, and the two differ by the version and by the
/// cherry fields. A reader that takes the first and refuses the second passes that test and drops every run
/// on somebody else's disk.
#[test]
fn a_th06_file_the_release_wrote_has_its_song_put_back() {
    picked_up_out_of(
        "th06-the-song-of-the-releases-file",
        as_the_release_wrote_it,
        "the file the released orb wrote",
    );
}

/// A run played to the card's chapter and given up there, with the file it left handed to `changed` before the
/// pick-up reads it — which is how the bytes of an orb that is not this one are reached from here — and then
/// picked up, holding the song to where that chapter had it.
///
/// **What is asserted is where the track ends up, not that a line was said.** A song put back at the track's
/// opening is a song the log calls put back and a listener calls started over, which is the whole of what a
/// run of th06 on a disk stands to lose: the chapter is [`SONG`] bytes into a track whose loop is millions of
/// bytes long, so the file's own position, the countdown that goes with it, and the buffer coming out are the
/// three things that say the music carried on from where the chapter left it.
///
/// `whose` names the file the failures are about, there being more than one kind of older file.
fn picked_up_out_of(name: &str, changed: fn(&[u8]) -> Vec<u8>, whose: &str) {
    in_its_own_process(|| {
        let game = Fake::attach(name, the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.streams_its_song(SONG);
        let loops_at = game.loop_point();
        game.in_a_pointdevice_run();
        game.frames_until("the chapter the card is", 900, || {
            log.said(&format!("at frame {CARD_STARTS}"))
        });
        game.one_frame_to_drain_the_log();

        game.dies();
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

        // The file the run left, written back as the other orb's own bytes.
        //
        // **In the simulated Windows' own files and not on this host's disk**, which is where a laid-out
        // game's writes go — see `orb_sim`'s file table. And waited for, the write being off the frame's
        // thread: a file asked for the frame the chapter was written is a file that is not there yet.
        let files = game.image().sim().files();
        let path = game
            .dir()
            .join("pointdevice_resume")
            .join("normal-reimu-a.msgpack");
        game.frames_until_a_thread("the chapter on the disk", || files.holds(&path));
        let bytes = files.get(&path).expect("the file the run left behind");
        files.put(&path, changed(&bytes));

        let from = log.written();
        game.picks_the_run_up();
        assert!(
            log.said_since(from, &format!("resume: the song picked up at {SONG}")),
            "{whose} had its song left alone, or put back somewhere other than where the chapter had it, \
             which is music taken away from a run of th06 already on a disk:\n  {}",
            log.lines().join("\n  "),
        );
        // And the track itself where the line says it is: the file read on from that place rather than from
        // the opening, the countdown short by the sound that was skipped, and the buffer coming out.
        let stream = game.stream_now();
        assert!(
            stream.position >= SONG,
            "the track was left at {} in the file, which is the song started over rather than picked up \
             at {SONG}",
            stream.position,
        );
        assert!(
            game.image().bytes_left() < loops_at - SONG as u32,
            "the countdown says the stream has the whole track ahead of it, which is a song playing from \
             its opening",
        );
        assert!(
            stream.playing,
            "the song was put back at {SONG} and nothing is coming out of the buffer, which is silence \
             where the chapter had its music",
        );
    });
}

/// The same file with `song_identity` and `song_sounding` taken out of its chapter, which is what the build
/// before this one wrote: the keys gone rather than written as nothing, a file that says nothing about them
/// being a different thing from one saying it does not know.
///
/// # Panics
/// Where the bytes are not a file with a chapter in it, which is a test that has changed what it writes.
fn without_the_songs_fields(bytes: &[u8]) -> Vec<u8> {
    use rmpv::Value;

    let named = |name: &str| Value::String(name.into());
    let mut whole = match rmpv::decode::read_value(&mut &bytes[..]) {
        Ok(Value::Map(fields)) => fields,
        other => panic!("not a map: {other:?}"),
    };
    let chapter = whole
        .iter_mut()
        .find(|(key, _)| *key == named("chapter"))
        .expect("the chapter of the file");
    let Value::Map(fields) = &mut chapter.1 else {
        panic!("the chapter is not a map: {:?}", chapter.1)
    };
    fields.retain(|(key, _)| *key != named("song_identity") && *key != named("song_sounding"));
    let mut written = Vec::new();
    rmpv::encode::write_value(&mut written, &Value::Map(whole)).expect("written");
    written
}

/// And the same file as **v0.10.0** wrote its own: the version it wrote, no generators, no cherry among the
/// numbers a stage starts with, and a chapter that ends at its song position.
///
/// Read off that release's `crates/orb-core/src/resume.rs` at the tag — see `game::th06::resume::SHIPPED`,
/// where the same shape is written down beside the number, and `the_file_the_release_wrote_is_read_and_offered`,
/// which holds the reader to it without a launch around it.
///
/// # Panics
/// Where the bytes are not a file with a chapter and a start in it, which is a test that has changed what it
/// writes.
fn as_the_release_wrote_it(bytes: &[u8]) -> Vec<u8> {
    use rmpv::Value;

    /// What v0.10.0's own `VERSION` is.
    const SHIPPED: u32 = 1;

    let named = |name: &str| Value::String(name.into());
    let mut whole = match rmpv::decode::read_value(&mut &bytes[..]) {
        Ok(Value::Map(fields)) => fields,
        other => panic!("not a map: {other:?}"),
    };
    for (key, value) in whole.iter_mut() {
        if *key == named("version") {
            *value = Value::from(SHIPPED);
        }
    }
    whole.retain(|(key, _)| *key != named("generators"));
    let mut without = |of: &str, keys: &[&str]| {
        let holder = whole
            .iter_mut()
            .find(|(key, _)| *key == named(of))
            .unwrap_or_else(|| panic!("the {of} of the file"));
        let Value::Map(fields) = &mut holder.1 else {
            panic!("the {of} is not a map: {:?}", holder.1)
        };
        fields.retain(|(key, _)| !keys.iter().any(|gone| *key == named(gone)));
    };
    without("start", &["cherry", "cherry_target", "cherry_plus"]);
    without("chapter", &["song_identity", "song_sounding"]);
    let mut written = Vec::new();
    rmpv::encode::write_value(&mut written, &Value::Map(whole)).expect("written");
    written
}

/// The countdown a refill's own line carries: what is left of the wav before the loop is taken.
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
fn the_refill_of_th06_says_what_it_is_feeding_the_buffer() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-refill", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        let before = game.loop_point() - game.stream_now().position as u32;
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
fn the_log_of_th06_tells_the_track_being_heard_from_another_once_per_change_of_it() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-track-heard", the_run(), |config| {
            config.log_level = LogLevel::Normal;
        });
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);

        let said = || {
            log.lines()
                .iter()
                .filter(|line| line.contains("sound: 0x"))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        };
        let streamed = said();
        assert_eq!(
            streamed.len(),
            1,
            "the track being heard was said other than once for the track that was on:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            streamed[0].contains("is playing at"),
            "the line does not say the track is playing and where it is:\n  {}",
            streamed[0],
        );

        // Another track on the same stream, which is a wave file with its own length and loop points: what the
        // line has to do is come out different.
        game.plays_the_stages_own_song();
        game.frames(APART);
        let both = said();
        assert_eq!(
            both.len(),
            2,
            "the track changing was not followed:\n  {}",
            log.lines().join("\n  "),
        );
        assert_ne!(
            both[0], both[1],
            "the two tracks were said with the same line, so nothing in the log tells them apart",
        );
    });
}
