//! **A resume puts its chapter's song back only into the track that song was in.**
//!
//! What a chapter writes down about its music is one number: where the sound audible as it began was in the
//! file the game streams from. `thbgm.dat` is one file holding every track, so that number is a position in
//! *that* file and it means the sound it points at only while the track playing is the one it was read from.
//!
//! **A stage's first chapter is picked up into the front end's own song**, which is what makes this reachable:
//! that chapter falls frames into the stage — `stage 6 chapter 1 (stage start) at frame 9` — and by then the
//! build has loaded the stage's two slots and its script has asked for neither, so what the stream holds is
//! still the record of the track the front end was playing. Seeking *that* stream to a position read off the
//! stage's own track hands the game whatever sound lies at those bytes, and starts the countdown that loops it
//! from a length belonging to another wav.
//!
//! Which is what a session of th07 was: a run picked up at stage 6's first chapter, its file holding
//! `song: 242508064`, and the landing saying `music: the track loops at 16545584, so 0 byte(s) left from
//! 242508064` — a position 242 MB into a 444 MB `thbgm.dat` put into a stream whose track ends 16 MB in. What
//! played was the fifth stage's boss theme.
//!
//! **What the sound itself does is not asserted here, and cannot be**: the bytes `orb_sim::Sound` serves are
//! one track's, based at nothing, so every track of the fake shares them and a position in one is a position
//! in all. Making that file hold the tracks at their own places is what would let a test hear the wrong one.

use crate::fake::th07::{FIRST_STAGE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames the stage runs before the chapter this is about: past the shortest a chapter may be, and far
/// enough in that the song has moved from where the stage started it.
const APART: u32 = 120;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// **And a chapter whose sound was nothing comes back to that nothing.**
///
/// The position of a stopped stream is where its sound *would* start, so putting it back and playing it makes
/// a silence audible — and a stage's own first chapter is taken in exactly that: it falls frames into the
/// stage, the song of the scene before it has been taken down, and the stage's own has not been asked for.
///
/// Which is what a launch of th07 heard: a run picked up at stage 6's first chapter played the **title
/// screen's own theme** for the five seconds until that stage's script started its song —
/// `resume: the song picked up at 705616 in the track's own file`, and 705616 is where the title's track was
/// stopped when the chapter was written.
///
/// **The restore has held a chapter to this all along** — `restore: this chapter had nothing audible; the
/// track asked for is stopped again`, and an e2e test of it in each game — and the resume owes the same. What
/// the fix for the resume's music before this one settled was a different half: the frames a playback feeds
/// are quiet, so the stage's opening does not play over and over while the buttons go in. Neither that fix nor
/// its test said anything about a chapter that had no sound.
#[test]
fn a_resume_of_th07_leaves_the_song_alone_where_the_chapter_had_no_sound() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-song-of-a-silent-chapter",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The stream there with its buffer stopped, which is the state a stage's first chapter is taken in:
        // the song of the scene before it taken down, and the stage's own not started.
        game.took_down_the_song_it_came_in_with(0);
        game.in_a_pointdevice_run();
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        game.gives_the_run_up();
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

/// **And puts it back where the track it was read from is the one playing**, which is the pick-up that has
/// always worked and the half a check on the track could take away: a chapter deep enough into a stage is
/// picked up with the stage's own song on the stream, and its position is a position in that song.
#[test]
fn a_resume_of_th07_puts_the_song_back_into_the_track_it_was_read_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-song-of-its-own-track",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        game.gives_the_run_up();
        let from = log.written();
        game.picks_the_run_up();

        assert!(
            log.said_since(from, "resume: the song picked up at"),
            "the chapter's own position was not put back into the track it was read from:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

#[test]
fn a_resume_of_th07_leaves_the_song_alone_where_another_track_is_playing() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-song-of-another-track",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The stage's own song playing, and a chapter of the stage written down while it does: the position
        // that chapter holds is one read off this track.
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // And picked up into the front end's own song, the stage's script not having asked for the stage's.
        game.gives_the_run_up();
        let from = log.written();
        game.picks_the_run_up_with_the_front_ends_song_on(FIRST_STAGE);

        assert!(
            !log.said_since(from, "resume: the song picked up at"),
            "the chapter's own position went into a stream playing another track:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            log.said_since(from, "resume: the song was left alone"),
            "the landing said nothing about the song it did not put back:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
