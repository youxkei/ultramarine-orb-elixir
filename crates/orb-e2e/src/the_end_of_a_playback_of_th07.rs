//! **A playback of th07 that has run out feeds nothing again, and the stage after it is where that was got
//! wrong.**
//!
//! What bounds a playback is the frame it stops feeding on, held beside the buttons, and what it is compared
//! against is the frame of the *stage*. Both are the stage's own numbering, so the two agree for as long as the
//! run stays in the stage it was picked up in: every frame the run reaches after the landing is past the frame
//! the feeding stopped on, and the playback answers nothing.
//!
//! **A stage boundary puts that frame back to nothing while the bound stays where it was.** The stage the run
//! walks into is registered with the buttons of the stage before it thrown away and the frame the feeding
//! stopped on left standing, so every frame of the new stage is below it — and the buttons the player plays
//! there are written down against those same frames as they are played. A frame reached twice is then a frame
//! the playback has an answer for: the chapter retried hands the run the buttons somebody played through it a
//! minute ago, one frame after another, and the run plays itself through the whole stretch it has already been
//! played through.
//!
//! Which is what a session of th07 was: a run picked up, played to the sixth stage, and one chapter of it
//! retried, after which it moved on its own and looked like a replay of something else being watched. The
//! game's own run kind had neither its demo nor its replay bit set — that word is orb's to clear rather than
//! to set — and what the log did say was one line, at the stage boundary rather than at the retry: the
//! playback fed frame 1 straight after frame 17863.
//!
//! **The music was held quiet on those frames too**, which is the other half of what a session hears: the read
//! takes the song down on every frame it feeds, so that a playback does not play the stage's opening over and
//! over for the seconds its buttons take. A stage of fed frames is a stage whose song is taken down on every
//! one of them. Nothing below asserts that: what the fake does to the sound across a stage boundary is not laid
//! out, and laying it out is a change to the fake rather than to what this file is about.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run moves on to, counted from zero the way everything above `Game` counts.
const NEXT: i32 = 1;

/// The frames a stage runs before the thing it is being run for: past the shortest a chapter may be, so the
/// stage the run is picked up in has a chapter of its own to be left at.
const APART: u32 = 120;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The word a held direction leaves — see the fake's own `MAP`. **A direction and not the bomb or the shot**:
/// what this test needs is a word the run acts on that no menu of the game and no question of orb's reads as an
/// answer, so that the same key can be held through the stage being played.
const UP: u16 = 0x10;

/// The frames the retried chapter is watched for, which is the whole of the stretch the run had already been
/// played through: a playback that has an answer for those frames feeds them one after another.
const WATCHED_FOR: u32 = APART;

/// **The song of the stage after a pick-up keeps playing**, which is the same bound heard rather than seen.
///
/// Every frame that is fed is a frame the music is held quiet on: a playback runs its whole burst of updates
/// inside one frame of the game, and the song the stage's build started would otherwise play its opening over
/// and over for the seconds the buttons take. So a stage of fed frames is a stage with its song taken down on
/// every one of them — and the frames of the stage after the landing are fed as soon as anything has written
/// them down, a frame the game reads twice being one the playback has an answer for by its second read.
#[test]
fn the_song_of_the_stage_after_a_playback_of_th07_keeps_playing() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-song-after-a-playback",
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
        game.picks_the_run_up();

        // **And the game slowing its own run down from here**, which is what makes a frame be read twice: the
        // first of those reads is what writes the frame down, so the second is the one a playback still holding
        // a bound has an answer for.
        game.slows_its_own_run_down();
        game.moves_on_to_the_stage(NEXT);
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said("stage 2 chapter 1 (stage start)")
        });
        // The song of the stage the run walked into: its script asks for the slot the build loaded, and the
        // buffer that stream is on answers as playing again — a transition leaves the song of the stage before
        // it taken down, so this is a stream on a buffer that had been released.
        game.its_stage_song_starts();
        orb_sim::Sound::started_again();
        assert!(
            game.stream_now().playing,
            "the stage's own song was not playing to begin with, so nothing here is about what stops it",
        );
        game.frames(APART);
        assert!(
            game.stream_now().playing,
            "the song of the stage after the landing was taken down, the frames of that stage being fed by \
             the playback that landed in the stage before it",
        );
    });
}

#[test]
fn a_playback_of_th07_that_has_landed_feeds_no_frame_of_the_stage_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-playback-past-a-stage-boundary",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A chapter of the first stage to leave the run at, deep enough into it that the frame the feeding
        // stops on is past every frame of the stage after it.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // The run given up and picked up: the buttons played in, the landing reached, and the keyboard the
        // player's again from there.
        game.gives_the_run_up();
        game.picks_the_run_up();

        // And on into the next stage, with a direction held through the whole of what is played there: the
        // words those frames are written down as are what a playback with an answer for them would feed.
        game.moves_on_to_the_stage(NEXT);
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said("stage 2 chapter 1 (stage start)")
        });
        game.keyboard().set(keys::UP, true);
        game.frames(APART);
        assert_eq!(
            game.word_the_run_acted_on(),
            UP,
            "the run did not act on the direction held through the stage, so the frames it is written down \
             for hold nothing to tell a fed frame by",
        );
        game.keyboard().set(keys::UP, false);
        game.frames_until("the keyboard let go of", 8, || {
            game.word_the_run_acted_on() == 0
        });

        // The chapter of that stage retried, which is what puts its frames back to ones the run has already
        // been played through.
        let from = log.written();
        game.dies();
        game.frames_until("the death seen", 8, || {
            log.said_since(from, "died in chapter 1")
        });
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said_since(from, "retry: the chapter again on the keyboard")
        });

        // Nothing held, so every word the run acts on from here is one it was handed: the buttons of the
        // frames it has already played, fed back by a playback that stopped at the stage before this one.
        for _ in 0..WATCHED_FOR {
            game.frame();
            assert_eq!(
                game.word_the_run_acted_on(),
                0,
                "the run acted on a word nothing was pressing after the chapter was retried, the playback \
                 that landed in the stage before this one having an answer for the frames of this one",
            );
        }
    });
}
