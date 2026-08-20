//! **An update of 妖々夢 is not a frame of its run, and a playback was bounded as though it were.**
//!
//! orb drives the updates itself to put a run back: it calls the game's own calc chain again and again inside
//! one frame of its own, feeding a frame's buttons to each, until the stage's clock reaches the frame the
//! chapter began on. What bounded that loop was a count of those updates against the frame it was aiming at,
//! and the two are not the same count for either of two reasons.
//!
//! **The one a session has shown is a clock that stopped.** A run picked up in stage 6 got `24179 update(s)
//! run; stage 6 frame 5900 ... written down as chapter 14 at frame 24119`, which is the frame it aimed at plus
//! the slack: 5959 of those updates played the run in, and the eighteen thousand after them went by with the
//! clock where it was, the run having died out. Bounded on the clock instead, the same playback gives up at
//! 5959 and the game is back in the player's hands fifteen seconds sooner.
//!
//! **The one the exe shows is the game slowing its own run down.** 0x42e186 divides the manager's own count of
//! updates by five and answers 3 to the one that leaves nothing over, which skips both of the run's clocks —
//! stage 5 has it and stage 6 does not, so a stage played through that takes as many updates again and a bound
//! of `at + 60` would run out with the run still going. That is what the e2e test below drives, no session
//! having shown it.
//!
//! **The bound that covers both without the arithmetic is the clock itself**: a playback whose stage clock has
//! not moved for as many updates in a row as this one allows is a playback that is not going anywhere, and one
//! whose clock is still moving is worth another update however many it has taken.

use crate::fake::th07::{Fake, SLOWED_EVERY};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bomb's own bit of the word 妖々夢's read builds, which is the button a run reads as an edge rather than
/// as a hold — see the fake's own `MAP`.
const BOMB: u16 = 0x2;

/// The frames between the stage's own start and the card, where a chapter is wanted and not a deep one: past
/// the shortest a chapter may be.
const APART: u32 = 120;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// How long the stage runs before the card, in frames of the host: with the game slowing its own run down, the
/// stage's clock reaches four fifths of them, so the chapter is deep enough that the updates a playback of it
/// takes are more than that clock plus the slack a stopped one is given.
const BEFORE_THE_CARD: u32 = 750;

/// **A press made on a frame the game does not advance the run on is still a press.**
///
/// That update stops at the manager, which answers 3 and takes the chain walk with it: the player's job and the
/// recorder's are below it, so neither the word the run acts on nor last frame's of it moves, and the update
/// that does advance the run reads the pair the press is an edge in. orb writes that pair at the read instead —
/// at priority 0, above the manager — so it writes on the updates the game skips too, and a pair written twice
/// for one frame holds the same word twice. The edge is then gone, and a bomb on that frame is a bomb the run
/// never fired.
///
/// **Which is a run that plays differently every time the phase of that skipping moves**, and it moves between
/// the frames somebody plays and the burst of updates a playback drives: the recorded run bombed and the
/// playback of it did not.
#[test]
fn a_press_of_th07_on_a_frame_the_run_was_not_advanced_on_is_still_an_edge() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-press-on-a-skipped-update",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.slows_its_own_run_down();

        // The bomb pressed and let go every other update, so the word each read produces differs from the one
        // before it — which is what makes a word written twice for one frame show.
        //
        // **What has to hold still is the word the run last acted on**, that being the half of the pair a press
        // is an edge *against*: the word it acts on is the last thing the keyboard said before the update that
        // advances the run, which is also the word that frame is written down as.
        let mut before: Option<(u32, u16)> = None;
        let mut skipped = 0;
        for update in 0..SLOWED_EVERY * 4 {
            game.keyboard().set(keys::X, update % 2 == 0);
            // The frame this update's own read is for, which is the clock as it stands before it: what the
            // update leaves it at is the frame the *next* read belongs to.
            let frame = game.state().stage_frames;
            game.frame();
            let acted_on = game.word_the_run_acted_on();
            if let Some((was, held)) = before
                && was == frame
            {
                // The same frame read twice is the update the game did not advance the run on.
                skipped += 1;
                assert_eq!(
                    acted_on, held,
                    "the word the run last acted on moved on an update that did not advance frame {frame}, \
                     so a press of {BOMB:#06x} on that frame is an edge or not by how many updates the game \
                     spent on it — and a playback drives them in a burst",
                );
            }
            before = Some((frame, acted_on));
        }
        assert!(skipped > 0, "the game advanced the run on every update");
    });
}

/// **A second playback in one session is watched from its own first frame.**
///
/// What watches one is kept beside the buttons: the frame last fed, so that a jump in them is said, and whether
/// this playback has already been found to have parted. Both are a playback's own, and a run given up and
/// picked up again is a second playback — one that starts at frame 0 again, so the frame last fed by the one
/// before it is a jump backwards that nothing jumped.
#[test]
fn a_second_playback_of_th07_in_one_session_is_watched_from_its_own_first_frame() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-picked-up-twice",
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

        // The run picked up, given up again and picked up a second time: the same file both times, so the two
        // playbacks are the same frames in the same order.
        game.gives_the_run_up();
        game.picks_the_run_up();
        game.gives_the_run_up();
        let from = log.written();
        game.picks_the_run_up();
        assert!(
            !log.said_since(from, "resume: the playback was fed frame"),
            "the second playback was held against the first one's frames:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **Every frame's generator at the verbose level, played and played back alike**, which is what a run whose
/// file holds none of its own can be held against: one series is left by the session that played the run and
/// the other by the launch that picked it up, and the first frame they differ on is where the playback parted.
///
/// **At that level and not behind `self_check`**, which is where this began: that flag has a chapter fingerprint
/// every region it saved, and the snapshot a boundary takes went from 2ms to 204ms with it — a stutter at every
/// chapter of the run somebody is playing to leave the file. Two series of numbers cost log and nothing else.
#[test]
fn a_playback_of_th07_says_every_frames_generator_where_the_log_is_verbose() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-playback-generators",
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

        // The frames somebody played, said the same way: the two series are what hold a playback to the run it
        // is reproducing, so a run whose file carries no generator of its own is still held to one.
        let played = log
            .lines()
            .iter()
            .filter(|line| line.contains("played with the generator at"))
            .count();
        assert!(
            played >= at as usize,
            "the run said the generator on {played} of the {at} frames it was played for:\n  {}",
            log.lines().join("\n  "),
        );

        game.gives_the_run_up();
        game.picks_the_run_up();
        // One line per frame the playback moved the run onto, which is every frame of it but the one it began
        // standing on — that one is the stage's own start, and its generator is what the chapter line above
        // already carries.
        let said = log
            .lines()
            .iter()
            .filter(|line| line.contains("played back with the generator at"))
            .count();
        assert_eq!(
            said,
            at as usize - 1,
            "the playback said the generator on {said} of the {at} frames it played:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// A chapter deep in a stage is reached however many updates of a slowed run that takes.
#[test]
fn a_run_of_th07_is_put_back_at_a_chapter_deeper_than_the_updates_it_takes() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-deep-chapter-picked-up",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        // The game slowing its own run down, which is what makes the updates outnumber the frames.
        game.slows_its_own_run_down();
        game.frames(BEFORE_THE_CARD);

        // The chapter the card begins, deep in the stage: the file holds a frame of buttons for each of the
        // frames under it, and the playback has to run every one of them.
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", 120, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        assert!(
            at < BEFORE_THE_CARD,
            "the run was not slowed down: {at} frame(s) out of {BEFORE_THE_CARD} update(s)",
        );
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // And picked up again, which is the whole of what this is about: the playback takes more updates than
        // the frame it is aiming at, and it is the stage's clock standing still that says one is going nowhere.
        game.gives_the_run_up();
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );
        // **And said out loud on the way**, because the whole playback is one frame of the game: the window is
        // not redrawn and nothing on the screen moves for as long as it takes, which on a stage of 24119
        // frames is minutes. A log nobody can tell from a hang is what that looks like from outside.
        assert!(
            log.said(&format!("of {at} frame(s) played back")),
            "the playback said nothing on the way through {at} frames:\n  {}",
            log.lines().join("\n  "),
        );
        // **And the generator with it**, which is the one number a playback can be held against by hand: the
        // periodic state line a run writes carries it too, so a run recorded in one session and played back in
        // another leaves two series of it in the same log, and the first pair that differs is where the
        // playback parted. That is how a run whose file holds no generator of its own is diagnosed at all.
        assert!(
            log.said("update(s) in, the generator at 0x"),
            "the playback said nothing about the generator:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            game.state().stage_frames,
            at,
            "the run came back at another frame of the stage:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
