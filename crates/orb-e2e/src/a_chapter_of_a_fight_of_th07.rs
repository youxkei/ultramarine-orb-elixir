//! **The chapters a fight of 妖々夢 causes: the boss arriving, each attack it moves to, and the waves handed
//! back when a midboss goes down.**
//!
//! Which fight it is comes out of the *music*: an STD names two songs per stage and the game plays the second
//! for the boss the stage ends with, while the timeline says nothing — that reading was made off 紅魔郷 in
//! `a_chapter_of_the_stages_own_boss.rs`, and what this asks is that 妖々夢's own fields carry it. The
//! manager's boss slot, the timer each attack puts back to nothing, the flag and number a card is declared as,
//! and the record the streaming source is reading: those four are what a fight of this game is, and a chapter
//! at every boundary of one is what makes the mode worth having in a fight at all.
//!
//! **And that the stage's own boss going down is not a boundary**: the stage is over, and a chapter whose
//! first act is that the fight is already won has nothing to retry.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The cards the two fights put up, by the record each has of its own, named the way 妖々夢 names cards.
const MIDBOSS_CARD: usize = 96;
const MIDBOSS_CARD_NAME: &str = "幽夢「猫の見た夢」";
const BOSS_CARD: usize = 104;
const BOSS_CARD_NAME: &str = "反魂「二重の劫火」";
/// And the card the fight ends on, which is a card of the game's own like the two above: what makes it the last
/// one is the defeat that follows it rather than anything about the number.
const BOSS_LAST_CARD: usize = 105;

/// The frames between one part of a fight and the next: past the shortest a chapter may be, so that each part
/// is a chapter of its own rather than one folded into the chapter before it.
const APART: u32 = 120;

/// A 完全無欠モード run of this game with its stage's own track streaming, which is what makes a boss arriving
/// under it a *midboss*: the music is what says which fight it is.
fn fighting(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game
}

/// Waits for the chapter `number` and what caused it, whatever frame it fell on.
///
/// The frame is the fake's own clock and says nothing a reader of this file wants — the parts of the fight are
/// driven from here rather than off a timeline, so which frame each landed on is this file's own arithmetic.
/// What the fight decides is the cause, and that is what is read back.
fn chapter(game: &Fake, number: u32, cause: &str) {
    let log = game.log();
    game.frames_until(&format!("chapter {number}, {cause}"), APART, || {
        log.lines().iter().any(|line| {
            line.contains(&format!("chapter {number} at frame")) && line.contains(cause)
        })
    });
}

/// Runs the frames between one part of a fight and the next.
fn on(game: &Fake) {
    game.frames(APART);
}

#[test]
fn a_fight_of_th07_is_a_chapter_at_its_arrival_at_every_attack_and_at_the_waves_after_it() {
    in_its_own_process(|| {
        let game = fighting("th07-a-fight");
        let log = game.log();

        // The midboss arriving, which is its first attack starting.
        on(&game);
        game.the_boss_arrives();
        chapter(&game, 2, "a midboss nonspell");

        // The card it puts up, which is a fresh thing to fight: the flag and the number the manager declares
        // it by are what name the chapter.
        on(&game);
        game.a_card_starts(MIDBOSS_CARD, MIDBOSS_CARD_NAME);
        chapter(&game, 3, "a midboss spellcard");
        assert_eq!(
            game.state().spellcard,
            Some(MIDBOSS_CARD as u32),
            "the chapter began without the card that named it",
        );

        // The attack after it, which the game says by putting the boss's own timer back to nothing — the card
        // ending says nothing on its own, and here it is the same frame.
        on(&game);
        game.the_card_is_over();
        game.the_attack_changes();
        chapter(&game, 4, "a midboss nonspell");
        assert_eq!(
            game.state().spellcard,
            None,
            "the attack that began the chapter was named on the frame it began",
        );

        // The midboss beaten, which hands the stage back: the waves that follow want a chapter of their own,
        // and the fight's own end is the only boundary between them and it.
        on(&game);
        game.the_boss_is_beaten();
        chapter(&game, 5, "the midboss was beaten");
        assert!(
            !game.state().boss_present,
            "the fight orb called beaten is still going on",
        );

        // And the fight the stage ends with, which is the boss arriving with the *other* song under it: the
        // chapters it causes are the boss's and not a midboss's, which is the music saying which fight it is.
        on(&game);
        game.the_stages_own_boss_arrives();
        chapter(&game, 6, "a boss nonspell");

        on(&game);
        game.a_card_starts(BOSS_CARD, BOSS_CARD_NAME);
        chapter(&game, 7, "a boss spellcard");

        // **The last card of the fight, declared with the boss all but dead**, which is the shape a launch of
        // 妖々夢 was found in: `boss_life=46` a moment before, a card declared, and the boss gone 42 frames
        // later. A chapter for it is a chapter whose whole content is the defeat.
        on(&game);
        game.a_card_starts(BOSS_LAST_CARD, BOSS_CARD_NAME);
        chapter(&game, 8, "a boss spellcard");

        // The boss going down, which is *not* a boundary of its own: nothing of the stage is left to retry.
        let beaten = log.written();
        game.the_card_is_over();
        game.the_boss_is_beaten();
        game.frames(APART);
        assert!(
            !log.said_since(beaten, "chapter 9 at frame"),
            "the stage's own boss going down began a chapter with nothing to retry:\n  {}",
            log.lines().join("\n  ")
        );

        // **And the chapter the defeat closed is not what a retry goes back to**, which is what that launch
        // was left with: a hit taken in the frames the boss is going down offered the card it had just been
        // killed on, and going back there is going back to the defeat. What is offered is the chapter before
        // it, the last one with a fight in it.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter"));
        let died = log
            .lines()
            .iter()
            .rev()
            .find(|line| line.contains("died in chapter"))
            .cloned()
            .unwrap_or_default();
        assert!(
            died.contains("died in chapter 7"),
            "the chapter the defeat closed is the one a retry goes back to: {died}",
        );
    });
}

/// And **a midboss is a midboss whatever the stage's start was snapshotted under**, which the two songs a
/// stage's own data names are what settles: the game plays the second of them for the fight the stage ends with,
/// so a fight under the *first* of them is a midboss however the stage began.
///
/// Reached the way a launch of 妖々夢 reached it: a stage's first chapter is taken frames into the stage, where
/// the track the stream is on is still the song the stage was come into with — the stage's own starting after
/// it. Read off the track at the stage's start instead, every fight of that stage is the stage's own boss, its
/// chapters keep a music they should rewind, and the retry menu offers them under the wrong name. Watched on
/// 妖々夢's stage 6, whose midboss came out as `a boss nonspell, boss, music=keep`.
#[test]
fn a_midboss_of_th07_is_a_midboss_whatever_the_stages_start_was_taken_under() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-midboss-after-another-song",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.took_down_the_song_it_came_in_with(0);
        game.in_a_pointdevice_run();

        // The stage's own song, starting after that first chapter was taken — and then the midboss under it.
        game.its_stage_song_starts();
        on(&game);
        game.the_boss_arrives();
        game.frames_until("the chapter the midboss's arrival is", APART, || {
            log.said("chapter 2 at frame")
        });

        // Which is a midboss's chapter, and one that puts that song back rather than leaving it playing: the
        // same song as the waves either side of it.
        let chapter_2 = log
            .lines()
            .iter()
            .find(|line| line.contains("chapter 2 at frame"))
            .cloned()
            .unwrap_or_default();
        assert!(
            chapter_2.contains("a midboss nonspell") && chapter_2.contains("music=rewind"),
            "the fight under the stage's own song was taken as the boss the stage ends with: {chapter_2}",
        );

        // And the fight the stage ends with is still told apart from it: the *second* of the two names.
        on(&game);
        game.the_boss_is_beaten();
        chapter(&game, 3, "the midboss was beaten");
        on(&game);
        game.the_stages_own_boss_arrives();
        chapter(&game, 4, "a boss nonspell");
    });
}

/// **And a chapter of the fight's second phase keeps its music like every other of that fight.**
///
/// The song a second phase brings is a third one, and its name is the game's own rather than its stage's — see
/// `Th07`'s `SECOND_PHASE_SONG`. Read against the two the stage's data carries it is neither the stage's song nor
/// the fight's, so a chapter taken there is one whose music a restore *rewinds* while the chapters either side of
/// it are left playing.
///
/// A launch of 妖々夢 is what that cost: retrying a chapter of its sixth stage's second phase, the song was heard
/// looping over a stretch of itself and then still playing at the title — the restore having put back a stream
/// the second phase had already replaced.
#[test]
fn a_chapter_of_th07s_second_phase_keeps_its_music() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-second-phase",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The fight the stage ends with, whose chapters are the ones left playing.
        on(&game);
        game.the_stages_own_boss_arrives();
        chapter(&game, 2, "a boss nonspell");
        let first_phase = log
            .lines()
            .iter()
            .find(|line| line.contains("chapter 2 at frame"))
            .cloned()
            .unwrap_or_default();
        assert!(
            first_phase.contains("music=keep"),
            "the fight's own song was rewound with a chapter of its fight: {first_phase}",
        );

        // And its second phase, which is the third song: the same fight, so the same answer.
        on(&game);
        game.its_second_phase_begins();
        game.the_attack_changes();
        chapter(&game, 3, "a boss nonspell");
        let second_phase = log
            .lines()
            .iter()
            .find(|line| line.contains("chapter 3 at frame"))
            .cloned()
            .unwrap_or_default();
        assert!(
            second_phase.contains("music=keep"),
            "a chapter of the second phase rewinds a song the chapters around it leave playing: \
             {second_phase}",
        );
    });
}

/// **And a chapter left playing leaves the stream the game is playing through alone**, wherever that stream now
/// is.
///
/// A track change deletes the stream and its source and makes both again, so the pair the player names after one
/// is not the pair it named before. What a restore holds back is the ranges the sound was in *when the chapter
/// was taken*: read from there, the stream the game is playing through now is not held back at all, and the
/// snapshot's bytes go over its own — the countdown the loop is taken on among them.
///
/// Which is heard as the song going round a stretch of itself: the countdown runs out where the file is not, so
/// no loop of the track is taken and the buffer plays its own contents over. A launch of 妖々夢 retrying a
/// chapter of its sixth stage's fight had exactly that, and the song still playing at the title after it.
#[test]
fn a_chapter_of_th07_left_playing_leaves_the_stream_it_is_playing_through() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-stream-elsewhere",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A chapter of the fight, taken with the fight's own song playing: one left playing rather than rewound.
        on(&game);
        game.the_stages_own_boss_arrives();
        chapter(&game, 2, "a boss nonspell");

        // And then the second phase, whose stream is a pair of objects made somewhere else. The countdown that
        // one is on is what a restore must not touch.
        game.its_second_phase_begins();
        let playing_through = game.the_stream_its_player_names();
        assert!(
            playing_through.is_some(),
            "the second phase left the player naming no stream",
        );

        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 2"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        assert_eq!(
            game.the_stream_its_player_names(),
            playing_through,
            "the restore did not leave the stream the game is playing through as it was:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And a card that names itself a frame after the attack it belongs to began: the boss's timer resets first
/// and the card is declared after, so the chapter is already there and called a nonspell. What arrives late is
/// the name of the attack and not another attack, so the chapter takes the name — which is what the retry menu
/// then offers it as.
#[test]
fn a_card_of_th07_declared_after_its_attack_names_the_chapter_that_attack_began() {
    in_its_own_process(|| {
        let game = fighting("th07-a-late-card");
        let log = game.log();

        on(&game);
        game.the_boss_arrives();
        chapter(&game, 2, "a midboss nonspell");

        // The attack after it, with no name on the frame it began.
        on(&game);
        game.the_attack_changes();
        chapter(&game, 3, "a midboss nonspell");

        // And the name a frame later, inside the frames the shortest a chapter may be would refuse a second
        // chapter over: the chapter that is already there takes it.
        game.a_card_starts(MIDBOSS_CARD, MIDBOSS_CARD_NAME);
        game.frames_until("the card's name", 8, || {
            log.said("stage 1 chapter 3 at frame")
                && log.said("the attack it began has a name after all")
        });
        assert_eq!(
            game.state().spellcard,
            Some(MIDBOSS_CARD as u32),
            "the card that arrived late is not the one up",
        );
        assert!(
            !log.said("chapter 4 at frame"),
            "a second chapter began for an attack that only got its name late:\n  {}",
            log.lines().join("\n  ")
        );

        // 被弾, and the retry menu offers the chapter as the card it turned out to be rather than as the
        // nonspell it arrived as — which is what taking the name is *for*.
        game.dies();
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
