//! **A chapter caused by the stage's own boss, and one the card it names late is folded into.**
//!
//! The two are one mechanism seen twice. What tells the fight the stage *ends* with from its midboss is
//! the music and nothing else — an STD names the stage's song and the boss's, and the game plays the second
//! for the boss it ends with, while the timeline says nothing, stage 3 parking on the same wait for its
//! midboss. So the same three attacks fought with the two songs laid out are the boss's chapters and fought
//! without them are the midboss's, and that is the whole difference between these two e2e tests.
//!
//! And the card whose name arrives late. **The boss's timer resets first and the spellcard is declared
//! after**, and where those fall on different updates the chapter is already there and called a nonspell —
//! measured on Patchouli's first card and Flandre's last, in stage 7. What arrives late is the name of the
//! attack and not another attack, so the chapter takes the name rather than a second chapter starting for
//! it, which the shortest a chapter may be would refuse anyway.

use crate::fake::th06::{
    Fake, STAGE_BOSS_ARRIVES, STAGE_BOSS_ATTACK_CHANGES, STAGE_BOSS_CARD, STAGE_BOSS_CARD_STARTS,
    STAGE_BOSS_LATE_CARD, STAGE_BOSS_NAMES_ITS_CARD, the_run,
};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;

/// A pointdevice run whose stage-ending fight is fought out, with the songs laid out or not.
///
/// The song is streamed either way, so that the stage takes its first chapter as soon as it has settled
/// rather than spending the whole of `MUSIC_WAIT_FRAMES` on a track that is never coming. What the second
/// argument decides is whether the *boss's* song exists at all, which is what these two e2e tests differ
/// by.
fn fighting(name: &str, two_songs: bool) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.streams_its_song(0);
    if two_songs {
        game.plays_its_songs();
    }
    game.fights_its_boss_out();
    game.in_a_pointdevice_run();
    game
}

/// Runs the fight out and reads back the cause of the chapter each of its two remaining attacks began, and
/// the name the second one ends up with.
///
/// `spell` and `nonspell` are how the log spells the two, which is the whole of what the music decides, and
/// `card` is the number of the chapter that card begins — one further on where the boss's own track began a
/// chapter of its own at [`STAGE_BOSS_ARRIVES`].
fn the_fight_reads_as(game: &Fake, card: u32, spell: &str, nonspell: &str, late_name: &str) {
    let log = game.log();

    // The card the fight puts up, which is a chapter of its own: the attack changed and a spellcard is up,
    // which is the pair that names it.
    game.frames_until("the card's chapter", STAGE_BOSS_CARD_STARTS + 120, || {
        log.said(&format!(
            "chapter {card} at frame {STAGE_BOSS_CARD_STARTS} \
             (script {STAGE_BOSS_CARD_STARTS}): {spell}"
        ))
    });
    assert_eq!(
        game.state().spellcard,
        Some(STAGE_BOSS_CARD as u32),
        "the chapter began without the card that named it",
    );

    // The attack after it, which arrives with no name: the timer reset is the whole of what says the fight
    // has moved on, so the chapter is a nonspell.
    let after = card + 1;
    game.frames_until(
        "the chapter the attack after the card is",
        STAGE_BOSS_ATTACK_CHANGES + 120,
        || {
            log.said(&format!(
                "chapter {after} at frame {STAGE_BOSS_ATTACK_CHANGES} \
                 (script {STAGE_BOSS_ATTACK_CHANGES}): {nonspell}"
            ))
        },
    );
    assert_eq!(
        game.state().spellcard,
        None,
        "the attack that began the chapter was named on the frame it began",
    );

    // And two frames later the name arrives. The chapter that is already there takes it — no further
    // chapter begins, which is what a second one for the same attack would be.
    game.frames_until("the card's name", STAGE_BOSS_NAMES_ITS_CARD + 120, || {
        log.said(&format!(
            "stage 1 chapter {after} at frame {STAGE_BOSS_ATTACK_CHANGES}: \
             the attack it began has a name after all"
        ))
    });
    assert_eq!(
        game.state().spellcard,
        Some(STAGE_BOSS_LATE_CARD as u32),
        "the card that arrived late is not the one up",
    );
    assert!(
        !log.said(&format!("chapter {} at frame", after + 1)),
        "a second chapter began for an attack that only got its name late:\n  {}",
        log.lines().join("\n  ")
    );

    // And the chapter is that card's from then on, which is what the name is *for*: 被弾, and the retry
    // menu offers the card rather than the nonspell the chapter began as.
    game.hit();
    game.frame();
    assert!(
        log.said(&format!("died in chapter {after}")),
        "the death was not noticed:\n  {}",
        log.lines().join("\n  ")
    );
    game.forget();
    game.frame();
    assert_eq!(
        game.says(late_name).len(),
        1,
        "the retry menu does not name the chapter for the card it turned out to be",
    );
}

/// With the two songs laid out, the fight the stage ends with is the boss's: its card is a boss spellcard
/// and the attack after it a boss nonspell.
#[test]
fn the_fight_the_boss_track_belongs_to_takes_boss_chapters() {
    in_its_own_process(|| {
        let game = fighting("the-boss-chapters", true);
        let log = game.log();

        // The track the fight the stage ends with brings, which is what makes every chapter below the
        // boss's rather than the midboss's. The fifth chapter of the stage: its own start, and then the
        // midboss's three.
        game.frames_until(
            "the fight the stage ends with",
            STAGE_BOSS_ARRIVES + 120,
            || {
                log.said(&format!(
                    "chapter 5 at frame {STAGE_BOSS_ARRIVES} (script {STAGE_BOSS_ARRIVES}): \
                 a boss nonspell"
                ))
            },
        );

        the_fight_reads_as(
            &game,
            6,
            "a boss spellcard",
            "a boss nonspell",
            "BOSS SPELL 2",
        );
    });
}

/// And with only the one song, the same three attacks are the midboss's: nothing else in the game says
/// which fight it is.
#[test]
fn the_same_fight_without_the_boss_track_takes_midboss_chapters() {
    in_its_own_process(|| {
        let game = fighting("the-midboss-chapters", false);
        let log = game.log();

        // No chapter at the frame the boss track would have started, there being no second song to start:
        // the fight underway is the one that has been underway since frame 400.
        game.frames_until(
            "the stage past where the boss track would start",
            STAGE_BOSS_ARRIVES + 120,
            || game.state().stage_frames > STAGE_BOSS_ARRIVES,
        );
        assert!(
            !log.said("a boss nonspell") && !log.said("a boss spellcard"),
            "a fight with one song under it took the stage boss's chapters:\n  {}",
            log.lines().join("\n  ")
        );

        // So the card is the stage's fifth chapter rather than its sixth, the frame the boss track would
        // have begun at having gone by without one.
        the_fight_reads_as(
            &game,
            5,
            "a midboss spellcard",
            "a midboss nonspell",
            "MIDBOSS SPELL 3",
        );
    });
}
