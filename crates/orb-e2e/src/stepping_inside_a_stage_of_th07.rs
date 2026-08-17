//! **The stepping keys over a replay of 妖々夢: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage.rs` asks of 紅魔郷, over this game's own chapters. Both keys go through
//! `Chapters::previous_start` and `next_start`, whose whole job is that a step lands **strictly** either side of
//! where the game is — so a press moves rather than staying put, and a frame or two past a boundary steps back
//! onto it.
//!
//! **Which chapters they land on is this game's own fight**, 妖々夢's midstage table being empty until a tuning
//! run fills it: the stage's own start and the three a fight has, read off the manager's boss slot and the timer
//! of an attack. So a step here is a restore of a snapshot of *this* game, and the frames it lands on are the
//! frames this file's own driving of that fight put the chapters at.
//!
//! Over a replay, because that is what a judging pass steps through: `--chapter-stepping` with
//! `--during-replay`, which is the launch that has nobody to answer a retry menu and every chapter kept all the
//! same.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;

/// One press of a stepping key with nothing else held, and the frame it left the game on.
///
/// # Panics
/// Where the step wrote no line saying where it held the game, which is a press that did nothing.
fn steps(game: &Fake, key: u8) -> u32 {
    let held = game.log().written();
    game.press(key);
    let said = game.log().lines().split_off(held).join("\n  ");
    assert!(
        said.contains("step: held"),
        "the press of {key:#04x} held the game nowhere:\n  {said}",
    );
    game.state().stage_frames
}

#[test]
fn a_step_inside_a_stage_of_th07_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-stepping-inside",
            Work::wandering(WORK_US),
            |config| {
                config.chapter_stepping = true;
                config.during_replay = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        let the_stages_own_start = game.state().stage_frames;

        // The fight, chapter by chapter, with the frame each began on kept: those are the frames a step has to
        // land on, and every one of them is where this game put a chapter rather than a number written here.
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        let the_midboss = game.state().stage_frames;

        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
        let the_card = game.state().stage_frames;

        game.frames(APART);
        game.the_card_is_over();
        game.the_attack_changes();
        game.frames_until("the chapter after the card", APART, || {
            log.said("chapter 4")
        });
        let after_the_card = game.state().stage_frames;

        // Held a little way into the last of them: a step back from inside a chapter goes to that chapter's own
        // start, which is what makes a press the way onto the boundary just passed.
        game.frames(APART / 2);
        let past = game.state().stage_frames;
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the third press does not stay on the frame
        // the second landed on.
        assert_eq!(steps(&game, BACK), after_the_card, "one press back");
        assert_eq!(steps(&game, BACK), the_card, "two presses back");
        assert_eq!(steps(&game, BACK), the_midboss, "three presses back");
        assert_eq!(
            steps(&game, BACK),
            the_stages_own_start,
            "four presses back",
        );

        // And the press after that, which has nowhere left to go inside the stage: the stage's own start is what
        // lies before its first boundary, and the restore has already arrived there.
        game.press(BACK);
        assert!(
            log.said(&format!(
                "step: held at the stage's start (script {the_stages_own_start})"
            )),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            the_stages_own_start,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same three: what a step forward runs to is the next start where the stage
        // has one.
        assert_eq!(steps(&game, NEXT), the_midboss, "one press forward");
        assert_eq!(steps(&game, NEXT), the_card, "two presses forward");
        assert_eq!(steps(&game, NEXT), after_the_card, "three presses forward");
    });
}
