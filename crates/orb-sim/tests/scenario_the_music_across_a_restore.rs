//! **The music put back where a chapter had it, and the stream believing the file it was given.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine — two of them by ear, which is what the numbers beside them stand in for here.
//!
//! What the laid-out game brings is a stage that streams the two songs its data names —
//! `Fake::plays_its_songs`, the stage's own and the boss's — which is the whole of what the question
//! *which* chapters put their music back turns on.

mod fake;

use fake::th06::{CARD_STARTS, Fake, STAGE_BOSS_ARRIVES, the_run};
use fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen};
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
/// Where no such chapter was taken, with every chapter line: a scenario reading the music of a chapter
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
/// Measured over a resumed run: the song was left at the track's opening milliseconds with the position
/// that had been written down (**`song=5900628`**) never read — the restore was asked of the chapter's
/// *kind*, and a midboss is not one of the midstage kinds although the stage's own song plays through it.
/// It is asked of the song now, from the same place a retry asks it, so the two cannot disagree about a
/// chapter.
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
        assert!(
            game.log().said("(MIDBOSS SPELL 1) at frame"),
            "the chapter the run is picked up into was not written down:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The run given up and started again, and つづきから: the stage is built, the buttons the run
        // pressed are played back into it, and the song is put where that chapter had it.
        gives_the_run_up(&game);
        picks_the_run_up(&game);
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
        assert!(
            game.log()
                .said(&format!("music: the track loops at {loops_at}, so")),
            "orb did not say what it worked the countdown out to be:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// How far into the track the sound now audible begins, for the scenario above.
///
/// Well inside the file and past a buffer's worth of it, which is what the episode needed to happen: the
/// real one was **5,900,628** bytes into `th06_02.wav`.
const SONG: i32 = 300_000;

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

/// The same run started again and answered つづきから, which is what plays the buttons it pressed back into
/// the stage the game has just built.
fn picks_the_run_up(game: &Fake) {
    let log = game.log();
    game.frames_until("the title menu ready to act on a press", 300, || {
        let front = game.image().front_end_now();
        game.image().scene() == Scene::FrontEnd
            && front.screen == Screen::Title
            && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered again", || {
        game.image().front_end_now().screen == Screen::ShotType
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        game.image().front_end_now().acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "つづきから answered", || {
        log.said("resume: from where it stopped, answered on the keyboard")
    });
    game.frames_until("the run played back into place", 60, || {
        log.said("resume: the landing is")
    });
}

/// The track is rewound for the chapters that share the stage's theme and left playing through a boss's.
///
/// Verified by ear over a full 1→6 run: rewound for the midstage and the midboss, which share the stage
/// theme, and left playing through a boss-fight restore.
///
/// And where the track has changed since the snapshot, it is asked to start again rather than copied back —
/// measured on a stage 6 run: `restore: the track has changed since this snapshot; taking the music down`,
/// `music: stopped through the game`, `music: restarting bgm/th06_12.mid`, with `StopBGM` and then
/// `PlayAudio` given the path read out of the memory the restore had just put back. **That half is the real
/// game's**: taking the music down and starting it again are calls into 紅魔郷's own `StopBGM` and
/// `PlayAudio` at their own addresses, and there is no code at those addresses in a game laid out by hand.
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
