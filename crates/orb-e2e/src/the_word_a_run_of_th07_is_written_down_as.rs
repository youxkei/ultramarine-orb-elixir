//! **Which bits of a frame's read are the run's, and what one that is not does when it is fed back.**
//!
//! 妖々夢's read builds one word and the game puts it in two places: the word the read left, which its front
//! end and its manager read, and the word the run acts on, which the recorder copies out of the first. The
//! game's own replay only ever writes the second — so every bit of a watched replay's word is the run's, and
//! the keyboard's menu button still opens the pause menu over it.
//!
//! **A resume of orb's goes in through the read instead**, and the manager takes the pause off exactly the word
//! that read leaves. So a menu button written down as part of a run is the pause menu opening in the middle of
//! a playback — and while it is up the manager runs no job of the run, so the frames of buttons after it are
//! fed into a game whose clocks are standing still.
//!
//! Which is a run picked up and not put back, and the e2e test below is a run that was paused where somebody
//! playing pauses: 完全無欠モード's own retry is an item of that menu, so every chapter retried is a run whose
//! frames had the menu open on them.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between the stage's own start, the pause, and the card: past the shortest a chapter may be.
const APART: u32 = 120;

/// A run that was paused part way through a stage is put back at its chapter all the same.
#[test]
fn a_run_of_th07_that_was_paused_is_put_back_at_its_chapter() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-paused-then-picked-up",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);

        // The game's own pause menu, put up the way somebody playing puts it up. What it does to the run is
        // stop it: neither clock moves while it is up, so the frames it is open on are one frame of the run.
        let stopped = game.state().stage_frames;
        game.press(keys::ESCAPE);
        assert!(
            game.state().paused,
            "the menu button did not put the menu up"
        );
        game.frames(APART);
        assert_eq!(
            game.state().stage_frames,
            stopped,
            "the stage went on with the game's own menu up",
        );

        // And 続ける, its own first item: the flag down and the run running again.
        game.press(keys::Z);
        assert!(!game.state().paused, "the menu did not go down on a decide");
        game.frames(APART);
        assert!(
            game.state().stage_frames > stopped,
            "the stage did not go on after the menu went down",
        );

        // Then a chapter with those frames behind it, and the file it is written in.
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // The run picked up out of it, which is where a word that is not the run's tells: the playback goes in
        // through the read, so the menu button of the frame that was paused would put that menu up again.
        game.gives_the_run_up();
        game.picks_the_run_up();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            !game.state().paused,
            "the run came back with the game's own menu up",
        );
    });
}
