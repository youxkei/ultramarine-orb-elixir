//! **The chapters a fight of th06nc causes: the boss arriving, each attack it moves to, and the waves handed
//! back when a midboss goes down** — `a_chapter_of_a_fight_of_th06c.rs` over this game's fight.
//!
//! **What says the fight moved on is the boss's own attack timer going back to nought**, `enemy::TIMER` in
//! `th06nc/mod.rs`, as it is on th06c: a card ending says nothing of its own, so the nonspell after one is
//! the timer and nothing else. A launch of the real game that read no timer had only the boss's first
//! nonspell for a chapter — its stage 2 fight `ch6 a boss nonspell` and then `ch7`, `ch8` and `ch9` all a
//! boss spellcard, where th06c's reads nonspell, card, nonspell, card — and a death in the nonspell after a
//! card went back to that card's start.
//!
//! **The fight is this laid-out game's own script**, [`MIDBOSS_ARRIVES`], [`CARD_STARTS`], [`ATTACK_CHANGES`]
//! and [`MIDBOSS_BEATEN`], all of it before the first row of stage 1's midstage table.

use crate::fake::th06nc::{ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, Work, in_its_own_process, the_frame_chapter_began_on};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames between one part of a fight and the next, which is how long a chapter is waited for.
const APART: u32 = 120;

/// How many frames the laid-out fight's own boundaries are waited for over: past the last of them.
const THE_FIGHTS_FRAMES: u32 = MIDBOSS_BEATEN + APART;

/// A Pointdevice mode run of this game in its first stage with nothing fired at the player — a death is a
/// thing the test takes where it means to.
fn fighting(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game
}

/// Waits for chapter `number` of the first stage to begin, caused by `cause`.
fn chapter(game: &Fake, number: u32, cause: &str) {
    let log = game.log();
    let began = format!("stage 1 chapter {number} at frame");
    game.frames_until(
        &format!("chapter {number}, {cause}"),
        THE_FIGHTS_FRAMES,
        || {
            log.lines()
                .iter()
                .any(|line| line.contains(&began) && line.contains(cause))
        },
    );
}

#[test]
fn a_fight_of_th06nc_is_a_chapter_at_its_arrival_at_every_attack_and_at_the_waves_after_it() {
    in_its_own_process(|| {
        let game = fighting("th06nc-a-fight");

        chapter(&game, 2, "a midboss nonspell");
        assert!(
            game.state().stage_frames >= MIDBOSS_ARRIVES,
            "the midboss's chapter began before the script put it up",
        );
        chapter(&game, 3, "a midboss spellcard");
        assert!(
            game.state().stage_frames >= CARD_STARTS,
            "the card's chapter began before the script put it up",
        );
        // The attack after the card, which the game says by putting the boss's timer back to nothing.
        chapter(&game, 4, "a midboss nonspell");
        assert!(
            game.state().stage_frames >= ATTACK_CHANGES,
            "the attack's chapter began before the script moved the fight on",
        );
        chapter(&game, 5, "the midboss was beaten");
        assert!(
            !game.state().boss_present,
            "the fight orb called beaten is still going on",
        );
    });
}

/// **A death in the nonspell after a card goes back to that nonspell and not to the card** — the chapter the
/// timer began, which the real game's run went back past to the card's start while no timer was read: the
/// retry puts the run back on the frame that chapter began on, with no card up.
#[test]
fn a_death_in_the_nonspell_after_a_card_of_th06nc_goes_back_to_that_nonspell() {
    in_its_own_process(|| {
        let game = fighting("th06nc-a-death-after-a-card");
        let log = game.log();

        chapter(&game, 4, "a midboss nonspell");
        let began = the_frame_chapter_began_on(&log.lines(), 1, 4);
        game.hit();
        game.frames_until("the death seen", APART, || log.said("died in chapter"));
        assert!(
            game.state().stage_frames < MIDBOSS_BEATEN,
            "the death came after the fight it was to be taken in was over",
        );
        let died = log
            .lines()
            .iter()
            .rev()
            .find(|line| line.contains("died in chapter"))
            .cloned()
            .unwrap_or_default();
        assert!(
            died.contains("died in chapter 4"),
            "a death in the nonspell after the card is not a death in that nonspell's chapter: {died}",
        );

        game.press_until(keys::Z, "the retry answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let back = game.state();
        assert!(
            (began..=began + 1).contains(&back.stage_frames),
            "the retry put the run on frame {} of the stage, where the nonspell's chapter began on frame {began}",
            back.stage_frames,
        );
        assert_eq!(
            back.spellcard, None,
            "the retry put the run back with a card up, which is the card's chapter and not the nonspell's",
        );
    });
}
