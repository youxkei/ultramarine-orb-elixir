//! **A playback of th06 that has run out feeds nothing again, and the stage after it is where that was got
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
//! **The same hook and the same record in both games**, which is why this file stands beside
//! `the_end_of_a_playback_of_th07.rs`: `resume::stage_begun` is reached through
//! `Hooks::stage_begun` — `Entered::OnTheStack` for th06 and `Entered::InEcx` for th07 — and the bound, the
//! buttons and the frame they are held against are the same three fields. So the session that showed this was
//! a session of th07 and nothing about it was th07's: this game walks into its stages the same way, and the
//! stage it walks into counts its frames from nothing the same way.
//!
//! **What is fed is read off the word the run acts on and off the music.** A fed frame is one nothing reads the
//! keyboard for, so a run acting on a direction nobody is holding is a run being handed its own buttons back;
//! and the read takes the song down on every frame it feeds, so that a playback does not play the stage's
//! opening over and over for the seconds its buttons take. Both are what a session of this game would show —
//! a run moving on its own, with the stage's music gone.

use crate::fake::th06::{CARD_STARTS, Fake, INVULNERABLE_AFTER_SPAWNING, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// How long each of this game's stages runs here: past the frame the card chapter starts on, so that the
/// chapter the run is left at is deep enough into the stage for every frame of the stage after it to be below
/// the bound, and not so far past it that the transition is a wait.
const STAGE_ENDS: u32 = CARD_STARTS + 120;

/// Which chapter the card the midboss puts up is, counting the stage's own start as the first: the two
/// before it are the stage's start and the midboss's arrival — see `pointdevice_run.rs`, where the three are
/// read off one stage.
const CARD_CHAPTER: u32 = 3;

/// The frames of the stage walked into that are played before the death, which is what the retry then puts
/// back: past the frames a stage's own start cannot be killed in, since a hit inside those is a hit the game
/// does nothing with — see `the_player_a_stage_starts.rs`.
const PLAYED_THROUGH: u32 = INVULNERABLE_AFTER_SPAWNING as u32 + 20;

/// The frames the retried chapter is watched for, all of them below the bound the landing left.
const WATCHED_FOR: u32 = 120;

/// The word a held direction leaves, as `Th06` has its buttons. **A direction and not the shot or the bomb**:
/// what this needs is a word the run acts on that no screen of the game and no question of orb's reads as an
/// answer, so the same key can be held through the whole of what is played.
const UP: u16 = 0x0010;

/// The frames the answer to a menu is let go of over, which is where `swallow_input` writes every button down
/// at once: the word on those frames is `u16::MAX` and is nothing a run acted on, so the watch starts after
/// them.
const LET_GO_OF_OVER: u32 = 4;

/// A pointdevice run played as far as the card's chapter, which is the chapter it is then left at.
fn played_to_the_cards_chapter(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.streams_its_song(0);
    game.stages_last(STAGE_ENDS);
    game.in_a_pointdevice_run();
    let log = game.log();
    game.frames_until("the card's chapter", CARD_STARTS + 60, || {
        log.said(&format!("chapter {CARD_CHAPTER} at frame {CARD_STARTS}"))
    });
    game.frames_until_a_thread("the chapter written down", || {
        log.said(&format!(
            "at frame {CARD_STARTS}, {CARD_STARTS} frame(s) of buttons"
        ))
    });
    game
}

/// The hit and then タイトルに戻る, which leaves the chapter behind for the next launch to offer.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
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
}

/// The run picked up and walked into the stage after the one it landed in, with a chapter of that stage
/// played through with a direction held.
///
/// The direction is what the frames of that stage are written down as: it is let go of at the end of this, so
/// that the retry menu is answered by a keyboard holding nothing and every word after it is one the run was
/// handed rather than one somebody pressed.
fn picked_up_and_played_into_the_stage_after_it(game: &Fake) {
    game.picks_the_run_up();
    game.frames_until("the stage after the landing", STAGE_ENDS * 2, || {
        game.state().stage == 1
    });
    game.keyboard().set(keys::UP, true);
    game.frames(PLAYED_THROUGH);
    assert_eq!(
        game.image().input_now(),
        UP,
        "the run did not act on the direction held through the stage, so the frames it is written down \
         for hold nothing to tell a fed frame by",
    );
    game.keyboard().set(keys::UP, false);
    game.frames_until("the keyboard let go of", LET_GO_OF_OVER, || {
        game.image().input_now() == 0
    });
}

/// And that chapter of it retried, which is what puts the run back on frames it has already been played
/// through.
fn the_chapter_retried(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.dies();
    game.frames_until("the death seen", READS_KEYS_AFTER, || {
        log.said_since(from, "died in chapter")
    });
    game.press_until(keys::Z, "the retry menu answered", || {
        log.said_since(from, "retry: the chapter again on the keyboard")
    });
    game.frames(LET_GO_OF_OVER);
}

/// **A playback that has landed feeds no frame of the stage after it**, which is the run moving on its own.
#[test]
fn a_playback_of_th06_that_has_landed_feeds_no_frame_of_the_stage_after_it() {
    in_its_own_process(|| {
        let game = played_to_the_cards_chapter("th06-a-playback-past-a-stage-boundary");
        gives_the_run_up(&game);
        picked_up_and_played_into_the_stage_after_it(&game);
        the_chapter_retried(&game);

        // Nothing held, so every word the run acts on from here is one it was handed: the buttons of the
        // frames it has already played, fed back by a playback that stopped in the stage before this one.
        for _ in 0..WATCHED_FOR {
            game.frame();
            assert_eq!(
                game.image().input_now(),
                0,
                "the run acted on a word nothing was pressing after the chapter was retried, the playback \
                 that landed in the stage before this one having an answer for the frames of this one",
            );
        }
    });
}

/// **And the song of that stage keeps playing**, which is the same bound heard rather than seen.
///
/// Every frame that is fed is a frame the music is held quiet on, so a stretch of fed frames is a stretch with
/// the stage's song stopped on every one of them — which is what a session hears where the run is moving on its
/// own.
#[test]
fn the_song_of_the_stage_after_a_playback_of_th06_keeps_playing() {
    in_its_own_process(|| {
        let game = played_to_the_cards_chapter("th06-the-song-after-a-playback");
        gives_the_run_up(&game);
        picked_up_and_played_into_the_stage_after_it(&game);
        assert!(
            game.stream_now().playing,
            "the song was not playing through the stage the run walked into, so nothing here is about what \
             stops it",
        );

        the_chapter_retried(&game);
        game.frames(WATCHED_FOR);
        assert!(
            game.stream_now().playing,
            "the song of the stage after the landing was taken down, the frames of that stage being fed by \
             the playback that landed in the stage before it",
        );
    });
}
