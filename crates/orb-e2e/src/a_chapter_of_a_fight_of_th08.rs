//! **The chapters a midboss of th08 causes: her arriving, each attack she moves to, and the waves handed back
//! when she goes down** — and a card that names itself after its attack began.
//!
//! What `a_chapter_of_a_fight_of_th07.rs` asks of that game, asked of this one. Which fight it is comes out of
//! the *music* here too: a stage's info block names its tracks, and the conversation before the stage's own
//! boss puts the second of them on the stream (0x434dce), while a midboss comes on under the first — see
//! `Th08::the_fights_own_song_is_playing`. The stage's own boss and her attacks are
//! `a_pointdevice_run_of_th08.rs`'s; what is left for this file is the fight the stage goes *on* after.
//!
//! **A run placed and its stage's clock stepped by hand**, the midboss put up and taken down between: what
//! this is about is the chapters, and the laid-out stage's own script has no midboss to bring on.

use crate::fake::th08::{Fake, THE_FIGHTS_TRACK};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames between one part of a fight and the next: past the shortest a chapter may be, so that each part
/// is a chapter of its own rather than one folded into the chapter before it.
const APART: u32 = 120;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The card the midboss puts up, by the record it has of its own: any card, what the test asks of it being
/// that it is one.
const A_CARD: usize = 20;

/// The track a stage comes on with, which is the first its info block names: the one a midboss fights to.
const THE_STAGES_OWN_TRACK: usize = 0;

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
    for _ in 0..FRAMES_FOR_A_BUILD {
        if log.said("stage 1 chapter 1 (stage start)") {
            return game;
        }
        a_frame_of_the_stage(&game);
    }
    panic!(
        "the stage's first chapter was never taken:\n  {}",
        log.lines().join("\n  ")
    );
}

/// **One frame of a stage placed by hand**, the run's clock stepped the way the stage's own jobs step it and
/// the script's left standing, so that no row of the stage's table comes up under what this is about.
fn a_frame_of_the_stage(game: &Fake) {
    let image = game.image();
    image.has_been_in_the_run_for(image.the_frames_the_run_has_been_going() + 1);
    game.frame();
}

/// **Her attack going on for `frames`**, its timer stepped from `from` a frame at a time as the game's own
/// steps it — a timer going back to nought being what says an attack began.
fn she_attacks_for(game: &Fake, from: u32, frames: u32) {
    for frame in from..from + frames {
        game.image().the_boss_has_attacked_for(0, frame as i32);
        a_frame_of_the_stage(game);
    }
}

/// And the stage going on without her.
fn the_stage_goes_on_for(game: &Fake, frames: u32) {
    for _ in 0..frames {
        a_frame_of_the_stage(game);
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
/// stage's end and takes none. Then the stage's own boss, to the track that is hers, which is a boss's chapter
/// and not a second midboss's.
#[test]
fn a_midboss_of_th08_takes_a_chapter_per_attack_and_hands_the_waves_back_when_she_goes() {
    in_its_own_process(|| {
        let game = in_the_first_stage("th08-a-midboss");

        the_stage_goes_on_for(&game, APART);
        game.image().a_boss_is_fighting(0, (192.0, 96.0), 1000);
        game.image()
            .the_stream_reads_the_track(THE_STAGES_OWN_TRACK);
        she_attacks_for(&game, 0, APART);
        let arriving = chapter(&game, 2);
        assert!(
            arriving.contains("a midboss nonspell"),
            "a fight under the stage's own track was taken as something other than a midboss: {arriving}",
        );

        game.image().a_card_is_up(A_CARD);
        she_attacks_for(&game, 0, APART);
        let her_card = chapter(&game, 3);
        assert!(
            her_card.contains("a midboss spellcard"),
            "the midboss's card took no chapter of its own: {her_card}",
        );

        game.image().no_card_is_up();
        game.image().no_boss_is_fighting();
        the_stage_goes_on_for(&game, APART);
        let handed_back = chapter(&game, 4);
        assert!(
            handed_back.contains("the midboss was beaten"),
            "her going down handed no chapter back to the waves: {handed_back}",
        );

        game.image().a_boss_is_fighting(0, (192.0, 96.0), 1000);
        game.image().the_stream_reads_the_track(THE_FIGHTS_TRACK);
        she_attacks_for(&game, 0, APART);
        let the_boss = chapter(&game, 5);
        assert!(
            the_boss.contains("a boss nonspell"),
            "the fight to the stage's second track was taken as another midboss: {the_boss}",
        );
    });
}

/// And a card that names itself a frame after the attack it belongs to began: the boss's timer resets first
/// and the card is declared after, so the chapter is already there and called a nonspell. What arrives late is
/// the name of the attack and not another attack, so the chapter takes the name — which is what the retry menu
/// then offers it as.
#[test]
fn a_card_of_th08_declared_after_its_attack_names_the_chapter_that_attack_began() {
    in_its_own_process(|| {
        let game = in_the_first_stage("th08-a-late-card");
        let log = game.log();

        the_stage_goes_on_for(&game, APART);
        game.image().a_boss_is_fighting(0, (192.0, 96.0), 1000);
        game.image()
            .the_stream_reads_the_track(THE_STAGES_OWN_TRACK);
        she_attacks_for(&game, 0, APART);
        assert!(
            chapter(&game, 2).contains("a midboss nonspell"),
            "the midboss's arrival took no chapter of her fight",
        );

        // The attack after it, with no name on the frame it began.
        she_attacks_for(&game, 0, 1);
        assert!(
            chapter(&game, 3).contains("a midboss nonspell"),
            "the attack after her first took no chapter",
        );

        // And the name a frame later, inside the frames the shortest a chapter may be would refuse a second
        // chapter over: the chapter that is already there takes it.
        game.image().a_card_is_up(A_CARD);
        she_attacks_for(&game, 1, APART);
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
        game.image().has_died(1.0);
        game.image().the_player_has_been_hit();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
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
