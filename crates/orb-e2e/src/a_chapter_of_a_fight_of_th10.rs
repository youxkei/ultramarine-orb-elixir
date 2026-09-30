//! **The chapters a midboss of th10 causes: her arriving, each attack she moves to, and the waves handed back
//! when she goes down** — and a card that names itself after its attack began.
//!
//! What `a_chapter_of_a_fight_of_th08.rs` asks of that game, asked of this one. Which fight it is comes out of
//! the *music* here as well: a fight is the stage's own boss where the record of `thbgm.fmt` the stream reads is
//! the one named by the song the stage's build loaded for her — `Th10::the_fights_own_song_is_playing` — which
//! the conversation before her puts on (0x416a91 through 0x416a9c), and a midboss comes on under the stage's own.
//!
//! **A run placed and its fights put up and taken down by hand**, the laid-out stage's own script left off:
//! what this is about is the chapters, and the frames they fall on are the test's and not that script's. The
//! same chapters come out of that script in `stepping_inside_a_stage_of_th10.rs`, read by a pass over a replay;
//! this is a run somebody plays.

use crate::fake::th10::{Fake, THE_BOSSS_TRACK, THE_MIDBOSSS_CARD};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames between one part of a fight and the next: past the shortest a chapter may be, so that each part
/// is a chapter of its own rather than one folded into the chapter before it.
const APART: u32 = 120;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 600;

/// Where a fight stands and the life it is given: anywhere in the field, and enough not to be beaten by
/// anything here.
const A_FIGHT_STANDS_AT: (f32, f32) = (0.0, 96.0);
const A_FIGHTS_LIFE: i32 = 10_000;

/// **A run placed in its first stage**, with the stage's first chapter taken and nothing fired at the player.
fn in_the_first_stage(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    let log = game.log();
    game.frames_until("the stage's first chapter", FRAMES_FOR_A_BUILD, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// **A fight come on**, its first attack's timer at nought.
fn a_fight_comes_on(game: &Fake) {
    game.image().a_fight_is_up(A_FIGHT_STANDS_AT, A_FIGHTS_LIFE);
}

/// **Her next attack begun**, its timer back at nought (0x41284c) — which is what says an attack began.
fn her_next_attack(game: &Fake) {
    game.image().the_fight_attacks_again();
}

/// **Her attack going on for `frames`**, its timer stepped a frame at a time as the enemy's update steps it
/// (0x40e572).
fn she_attacks_for(game: &Fake, frames: u32) {
    for _ in 0..frames {
        game.frame();
        game.image().the_fight_goes_on();
    }
}

/// The line the chapter `number` of stage 1 began with, which says what caused it.
///
/// # Panics
/// Where it has not begun.
fn chapter(game: &Fake, number: u32) -> String {
    let said = format!("stage 1 chapter {number} at frame");
    game.log()
        .lines()
        .into_iter()
        .find(|line| line.contains(&said))
        .unwrap_or_else(|| {
            panic!(
                "chapter {number} never began:\n  {}",
                game.log().lines().join("\n  ")
            )
        })
}

/// **A fight under the stage's own track is a midboss**, each of her attacks a chapter as a boss's are, and
/// **her going down hands the waves back as a chapter of their own** — where the stage's own boss going is the
/// stage's end and takes none. Then the stage's own boss, to the track the conversation before her puts on,
/// which is a boss's chapter and not a second midboss's.
#[test]
fn a_midboss_of_th10_takes_a_chapter_per_attack_and_hands_the_waves_back_when_she_goes() {
    in_its_own_process(|| {
        let game = in_the_first_stage("th10-a-midboss");

        game.frames(APART);
        a_fight_comes_on(&game);
        she_attacks_for(&game, APART);
        let arriving = chapter(&game, 2);
        assert!(
            arriving.contains("a midboss nonspell"),
            "a fight under the stage's own track was taken as something other than a midboss: {arriving}",
        );

        her_next_attack(&game);
        game.image().a_card_is_up(THE_MIDBOSSS_CARD);
        she_attacks_for(&game, APART);
        let her_card = chapter(&game, 3);
        assert!(
            her_card.contains("a midboss spellcard"),
            "the midboss's card took no chapter of its own: {her_card}",
        );

        game.image().no_card_is_up();
        game.image().the_fight_is_over();
        game.frames(APART);
        let handed_back = chapter(&game, 4);
        assert!(
            handed_back.contains("the midboss was beaten"),
            "her going down handed no chapter back to the waves: {handed_back}",
        );

        game.image().the_stream_plays_the_track(THE_BOSSS_TRACK);
        a_fight_comes_on(&game);
        she_attacks_for(&game, APART);
        let the_boss = chapter(&game, 5);
        assert!(
            the_boss.contains("a boss nonspell"),
            "the fight to the boss's own track was taken as another midboss: {the_boss}",
        );
    });
}

/// And a card that names itself a frame after the attack it belongs to began: the boss's timer resets first
/// and the card is declared after, so the chapter is already there and called a nonspell. What arrives late is
/// the name of the attack and not another attack, so the chapter takes the name — which is what the retry menu
/// then offers it as.
#[test]
fn a_card_of_th10_declared_after_its_attack_names_the_chapter_that_attack_began() {
    in_its_own_process(|| {
        let game = in_the_first_stage("th10-a-late-card");
        let log = game.log();

        game.frames(APART);
        a_fight_comes_on(&game);
        she_attacks_for(&game, APART);
        assert!(
            chapter(&game, 2).contains("a midboss nonspell"),
            "the midboss's arrival took no chapter of her fight",
        );

        // The attack after it, with no name on the frame it began.
        her_next_attack(&game);
        she_attacks_for(&game, 1);
        assert!(
            chapter(&game, 3).contains("a midboss nonspell"),
            "the attack after her first took no chapter",
        );

        // And the name a frame later, inside the frames the shortest a chapter may be would refuse a second
        // chapter over: the chapter that is already there takes it.
        game.image().a_card_is_up(THE_MIDBOSSS_CARD);
        she_attacks_for(&game, APART);
        assert!(
            log.said("stage 1 chapter 3 at frame")
                && log.said("the attack it began has a name after all"),
            "the chapter already there did not take the card's name:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("stage 1 chapter 4 at frame"),
            "a second chapter began for an attack that only got its name late:\n  {}",
            log.lines().join("\n  ")
        );

        // The hit, and the retry menu offers the chapter as the card it turned out to be rather than as the
        // nonspell it arrived as — which is what taking the name is *for*.
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 3"));
        game.frames(READS_KEYS_AFTER);
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("MIDBOSS SPELL 1").len(),
            1,
            "the retry menu does not offer the chapter as the card it turned out to be",
        );
    });
}
