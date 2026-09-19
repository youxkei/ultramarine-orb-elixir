//! **The chapters a fight of th06c causes: the boss arriving, each attack it moves to, and the waves handed
//! back when a midboss goes down.**
//!
//! **Which fight it is comes out of the track playing**, read against the track the stage's own start was
//! taken under. This game names none of its songs — `Th06c` leaves `Game::the_fights_own_song_is_playing`
//! at its default `None` — so what is left is `Game::music_identity`, the static its frame's own catch-up reads
//! (`THE_TRACK_PLAYING` in `th06c/mod.rs`): a fight under the stage's own track is a midboss, and a fight
//! under another is the boss the stage ends with. That is th06's reading, the one
//! `a_chapter_of_the_stages_own_boss.rs` makes of two songs an STD names, carried to its port. A launch of
//! the real game said the second half of it: `stage 2 chapter 8 at frame 8218 (script 5987): a boss
//! nonspell, boss, music=keep`, its stage's own boss being fought under a song the stage had not begun with.
//!
//! **The midboss is this laid-out game's own script** — [`MIDBOSS_ARRIVES`], [`CARD_STARTS`],
//! [`ATTACK_CHANGES`] and [`MIDBOSS_BEATEN`] — and the fight the stage ends with is driven from here, the
//! stage scripting no second fight. **So is the stage's own track**, which this laid-out game plays none of:
//! its run start leaves `-1` there, as the real one's `0x14002ccbb` does, and nothing after it in the fake
//! starts a song. Left at that, every fight is a midboss for the plainer reason that nothing playing is not
//! another track — see `chapter::a_track_other_than_the_stages` — and the midboss half of the reading would
//! pass without being asked.
//!
//! **And that the stage's own boss going down is not a boundary**: the stage is over, and a chapter whose
//! first act is that the fight is already won has nothing to retry.
//!
//! **Three of th07's five are not here.** `a_midboss_of_th07_is_a_midboss_whatever_the_stages_start_was_taken_under`
//! holds th07's reading of its stage data's two song names against the track a stage's first chapter was
//! taken under; this game's reading *is* that track, so the case that test guards is not one it can be
//! asked apart from the first test here. And the two over a fight's second phase and the stream it plays
//! through are about a stream a restore puts back and a third song th07 names — its `SECOND_PHASE_SONG`:
//! `Th06c::music` answers `None`, the block this game's track plays out of is among the ranges every restore
//! leaves where it finds them — `Th06c::audio_state` — and no third song of this game's has been read.

use crate::fake::th06c::{
    ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN, SCRIPTED_CARD,
};
use crate::fake::{Display, Launched, Work, in_its_own_process, the_frame_chapter_began_on};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The track the stage's own start is taken under, and the one the fight it ends with brings: any two tracks,
/// what the reading asks of them being that they are not the same one.
const THE_STAGES_OWN_TRACK: i32 = 1;
const THE_FIGHTS_OWN_TRACK: i32 = 2;

/// The cards the fight the stage ends with puts up, by the record each has of its own, and the name the game
/// puts on the plate for them. The last is a card like the one before it: what makes it the last is the
/// defeat that follows it rather than anything about the number.
const BOSS_CARD: u32 = 2;
const BOSS_CARD_NAME: &str = "THE BOSS'S CARD";
const BOSS_LAST_CARD: u32 = 3;

/// And the card the midboss names late, after the attack it belongs to began.
const LATE_CARD: u32 = 4;
const LATE_CARD_NAME: &str = "A CARD NAMED LATE";

/// What the fight the stage ends with is put up with, in the units the game keeps a boss's life in — which
/// this laid-out game writes over with its own on every update of a fight, so any number that is a life.
const THE_FIGHTS_LIFE: i32 = 1_000;

/// The frames between one part of a fight and the next: past the shortest a chapter may be, so that each part
/// is a chapter of its own rather than one folded into the chapter before it.
const APART: u32 = 120;

/// How many frames the laid-out fight's own boundaries are waited for over: past the last of them.
const THE_FIGHTS_FRAMES: u32 = MIDBOSS_BEATEN + APART;

/// A Pointdevice mode run of this game in its first stage, under the stage's own track, with nothing fired
/// at the player — a death is a thing each test here takes where it means to.
///
/// **The track after the run's start and not before**, which is the order the game has them in: the start
/// writes `-1` over whatever was playing.
fn fighting(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.image().plays_the_track(THE_STAGES_OWN_TRACK);
    game
}

/// The line chapter `number` began with, once it has: the one carrying what caused it and what a restore of
/// it does with the music.
fn the_line_of_chapter(game: &Fake, number: u32, cause: &str, within: u32) -> String {
    let log = game.log();
    let began = format!("stage 1 chapter {number} at frame");
    let found = || {
        log.lines()
            .into_iter()
            .find(|line| line.contains(&began) && line.contains(cause))
    };
    game.frames_until(&format!("chapter {number}, {cause}"), within, || {
        found().is_some()
    });
    found().unwrap_or_default()
}

/// Waits for the chapter `number` and what caused it, and asserts what a restore of it does with the music.
///
/// `music=rewind` for a chapter of the waves or of a midboss, which are played under the stage's own track;
/// `music=keep` for the fight the stage ends with, whose own song is left running across a retry — see
/// `Chapters::audio_for`.
fn chapter(game: &Fake, number: u32, cause: &str, music: &str, within: u32) {
    let line = the_line_of_chapter(game, number, cause, within);
    assert!(
        line.contains(music),
        "chapter {number} does not do with the music what its fight says: {line}",
    );
}

#[test]
fn a_fight_of_th06c_is_a_chapter_at_its_arrival_at_every_attack_and_at_the_waves_after_it() {
    in_its_own_process(|| {
        let game = fighting("th06c-a-fight");
        let log = game.log();

        // The midboss arriving, which is its first attack starting — under the stage's own track, which is
        // what makes it a midboss and its chapters ones whose song is put back.
        chapter(
            &game,
            2,
            "a midboss nonspell",
            "music=rewind",
            THE_FIGHTS_FRAMES,
        );
        assert!(
            game.state().stage_frames >= MIDBOSS_ARRIVES,
            "the midboss's chapter began before the script put it up",
        );

        // The card it puts up, which is a fresh thing to fight: the flag and the number the enemy manager
        // declares it by are what name the chapter.
        chapter(
            &game,
            3,
            "a midboss spellcard",
            "music=rewind",
            THE_FIGHTS_FRAMES,
        );
        assert!(
            game.state().stage_frames >= CARD_STARTS,
            "the card's chapter began before the script put it up",
        );
        assert_eq!(
            game.state().spellcard,
            Some(SCRIPTED_CARD),
            "the chapter began without the card that named it",
        );

        // The attack after it, which the game says by putting the boss's own timer back to nothing — the card
        // ending says nothing on its own, and here it is the same frame.
        chapter(
            &game,
            4,
            "a midboss nonspell",
            "music=rewind",
            THE_FIGHTS_FRAMES,
        );
        assert_eq!(
            game.state().spellcard,
            None,
            "the attack that began the chapter was named on the frame it began",
        );

        // The midboss beaten, which hands the stage back: the waves that follow want a chapter of their own,
        // and the fight's own end is the only boundary between them and it.
        chapter(
            &game,
            5,
            "the midboss was beaten",
            "music=rewind",
            THE_FIGHTS_FRAMES,
        );
        assert!(
            !game.state().boss_present,
            "the fight orb called beaten is still going on",
        );

        // And the fight the stage ends with, which is the boss arriving under a track the stage did not begin
        // with: the chapters it causes are the boss's and not a midboss's, and their song is left running.
        game.frames(APART);
        game.image().plays_the_track(THE_FIGHTS_OWN_TRACK);
        game.image().a_boss_is_fighting(THE_FIGHTS_LIFE, 0);
        chapter(&game, 6, "a boss nonspell", "music=keep", APART);

        // Its card, which is the attack changing and a card declared on the same frame, as the laid-out
        // midboss's own is.
        game.frames(APART);
        game.image().a_boss_is_fighting(THE_FIGHTS_LIFE, 0);
        game.image().starts_the_card(BOSS_CARD, BOSS_CARD_NAME);
        chapter(&game, 7, "a boss spellcard", "music=keep", APART);

        // **The last card of the fight, declared with the boss all but dead**, which is the shape a launch of
        // th07 was found in: a card declared, and the boss gone 42 frames later. A chapter for it is a chapter
        // whose whole content is the defeat.
        game.frames(APART);
        game.image().a_boss_is_fighting(THE_FIGHTS_LIFE, 0);
        game.image().starts_the_card(BOSS_LAST_CARD, BOSS_CARD_NAME);
        chapter(&game, 8, "a boss spellcard", "music=keep", APART);

        // The boss going down, which is *not* a boundary of its own: nothing of the stage is left to retry.
        let beaten = log.written();
        game.image().no_spellcard_is_running();
        game.image().no_boss_is_fighting();
        game.frames(APART);
        assert!(
            !log.said_since(beaten, "chapter 9 at frame"),
            "the stage's own boss going down began a chapter with nothing to retry:\n  {}",
            log.lines().join("\n  ")
        );

        // **And the chapter the defeat closed is not what a retry goes back to**: a hit taken in the frames
        // the boss is going down offered the card it had just been killed on, and going back there is going
        // back to the defeat. What is offered is the chapter before it, the last one with a fight in it.
        game.hit();
        game.frames_until("the death seen", APART, || log.said("died in chapter"));
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

/// **A death in the nonspell after a card goes back to that nonspell and not to the card**: the retry puts the
/// run back on the frame the nonspell's chapter began on, with no card up — th06nc's own run went back past it
/// to the card's start while that game's fight had no nonspell chapter after a card.
#[test]
fn a_death_in_the_nonspell_after_a_card_of_th06c_goes_back_to_that_nonspell() {
    in_its_own_process(|| {
        let game = fighting("th06c-a-death-after-a-card");
        let log = game.log();

        chapter(
            &game,
            4,
            "a midboss nonspell",
            "music=rewind",
            THE_FIGHTS_FRAMES,
        );
        let began = the_frame_chapter_began_on(&log.lines(), 1, 4);
        game.hit();
        game.frames_until("the death seen", APART, || log.said("died in chapter 4"));
        assert!(
            game.state().stage_frames < MIDBOSS_BEATEN,
            "the death came after the fight it was to be taken in was over",
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

/// And a card that names itself a frame after the attack it belongs to began: the boss's timer resets first
/// and the card is declared after, so the chapter is already there and called a nonspell. What arrives late
/// is the name of the attack and not another attack, so the chapter takes the name — which is what the retry
/// menu then offers it as.
///
/// The attack is the laid-out midboss's own after its card, at [`ATTACK_CHANGES`], and the name is given
/// it from here a frame later.
#[test]
fn a_card_of_th06c_declared_after_its_attack_names_the_chapter_that_attack_began() {
    in_its_own_process(|| {
        let game = fighting("th06c-a-late-card");
        let log = game.log();

        // The attack after the midboss's card, with no name on the frame it began: the fourth chapter, after
        // the stage's own start, the midboss's arrival and its card.
        the_line_of_chapter(&game, 4, "a midboss nonspell", THE_FIGHTS_FRAMES);
        let began = game.state().stage_frames;
        assert!(
            began >= ATTACK_CHANGES,
            "the attack's chapter began at frame {began}, before the script moved the fight on",
        );
        assert_eq!(
            game.state().spellcard,
            None,
            "the attack that began the chapter was named on the frame it began",
        );

        // And the name on the next frame, inside the frames the shortest a chapter may be would refuse a
        // second chapter over: the chapter that is already there takes it.
        game.image().starts_the_card(LATE_CARD, LATE_CARD_NAME);
        let named =
            format!("stage 1 chapter 4 at frame {began}: the attack it began has a name after all");
        game.frames_until("the card's name", 8, || log.said(&named));
        assert_eq!(
            game.state().spellcard,
            Some(LATE_CARD),
            "the card that arrived late is not the one up",
        );
        assert!(
            !log.said("chapter 5 at frame"),
            "a second chapter began for an attack that only got its name late:\n  {}",
            log.lines().join("\n  ")
        );

        // The hit, and the retry menu offers the chapter as the card it turned out to be rather than as the
        // nonspell it arrived as — which is what taking the name is *for*. The second of the midboss's cards,
        // the one at [`CARD_STARTS`] being the first.
        game.hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 4"));
        assert!(
            game.state().stage_frames < MIDBOSS_BEATEN,
            "the death came after the fight it was to be taken in was over",
        );
        assert_eq!(
            game.a_frame_that_says("MIDBOSS SPELL 2").len(),
            1,
            "the retry menu does not offer the chapter as the card it turned out to be",
        );
    });
}
