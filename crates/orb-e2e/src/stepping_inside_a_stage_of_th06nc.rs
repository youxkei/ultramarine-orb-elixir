//! **The stepping keys over th06nc: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage_of_th06c.rs` asks of that game, asked of this one. Both go through
//! `Chapters::previous_start` and `next_start`, whose whole job is that a step lands **strictly** either
//! side of where the game is — so a press moves rather than staying put, and a frame or two past a boundary
//! steps back onto it.
//!
//! **A pass over this game watches its own replay, and every playback of it reads as one**: `runtime::step`
//! asks for `during_replay && state.replay && state.in_run` together, and this game's `demo` answers false
//! whatever is playing — no flag telling the game's own attract playback from a replay somebody asked for
//! has been found, and both of `0x14004ede0`'s callers are the front end's. See
//! `Th06nc::say_what_a_playback_is_told_by_when_it_changes`, the instrument that watched the four bytes and
//! the state a playback could have been told by, and `th06nc/asm.md`'s *What is not established*.
//!
//! **And the boundaries are the fight's**, this game's midstage table being empty until a pass fills it:
//! the stage's own start, the midboss's arrival, the card it puts up and the frame that boss goes down on.
//! Which is what a pass has to step between before there is a table at all — see
//! `a_chapter_table_collected_for_th06nc`.
//!
//! **Four of them and not th06c's five**: that laid-out game's boss carries the count of the attack it is
//! on, so the frame its card hands over to a plain attack begins a chapter there; this one's
//! `a_boss_is_fighting` takes a life and no attack, and the fourth boundary here is the boss beaten.

use orb_config::LogLevel;

use crate::fake::th06nc::{CARD_STARTS, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;
/// And the one held with a step to leave the stage altogether, which is what
/// `moving_between_a_replays_stages` is about for the games that have it.
const ACROSS: u8 = orb_config::keys::SHIFT.0;

/// How long a stage lasts here: past the frame the fight's own boss goes down on, which is the last of
/// this stage's boundaries, and no further.
///
/// **Past that frame and not on it**, which is what a first run of this said: with the stage ending exactly
/// where the boss went down, the fourth chapter began on the stage's own last frame and the sixty frames
/// held after it were frames of the *next* stage — so a step back rewound that one's start and landed on
/// frame 9 where 900 was asked for.
const STAGE_FRAMES: u32 = MIDBOSS_BEATEN + 600;

/// **How far apart a chapter of this test's own is put from the boundary before it**, which is the shortest
/// a chapter may be with room to spare: two starts closer together than that are the same chapter.
const APART: u32 = 120;

/// **The fight this stage ends with, as the stage's own boss**: a life for it to be fought with, and a
/// track — any track, the laid-out stage playing none — which is what makes it the stage's own rather than
/// another midboss.
const THE_FIGHTS_LIFE: i32 = 1_000;
const THE_FIGHTS_OWN_TRACK: i32 = 5;

/// How many frames a wait here is given.
const FRAMES: u32 = STAGE_FRAMES * 3;

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

/// The launch a judging pass is: the stepping keys on over a replay, with nothing fired at the player — a
/// death would end the run this is stepping around, and a replay is no run to offer a retry for anyway.
fn judging(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_stepping = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.watches_a_replay_of_the_stage(0);
    game.frames_until("the stage's own first chapter", FRAMES, || {
        game.log().said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The four boundaries of this stage's fight, each waited for and the frame it began on kept.
fn the_boundaries(game: &Fake) -> (u32, u32, u32, u32) {
    let log = game.log();
    let the_stages_own_start = game.state().stage_frames;
    game.frames_until("the midboss's chapter", FRAMES, || log.said("chapter 2"));
    let the_midboss = game.state().stage_frames;
    game.frames_until("the card's chapter", FRAMES, || log.said("chapter 3"));
    let the_card = game.state().stage_frames;
    game.frames_until("the chapter after the card", FRAMES, || {
        log.said("chapter 4")
    });
    let after_the_card = game.state().stage_frames;
    // The frames the fight's own script puts them at, which is what says the waits above landed where this
    // game divides the stage rather than wherever the waiting stopped.
    assert!(
        the_midboss >= MIDBOSS_ARRIVES && the_card >= CARD_STARTS,
        "the fight's chapters did not fall where its script puts them: {the_midboss}, {the_card}",
    );
    (the_stages_own_start, the_midboss, the_card, after_the_card)
}

#[test]
fn a_step_inside_a_stage_of_th06nc_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = judging("th06nc-stepping-inside");
        let log = game.log();
        let (start, midboss, card, after) = the_boundaries(&game);

        // Held a little way past the last of them: a step back from inside a chapter goes to that chapter's
        // own start, which is what makes a press the way onto the boundary just passed.
        game.frames(60);
        let past = game.state().stage_frames;
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the second press does not stay on the
        // frame the first landed on.
        assert_eq!(steps(&game, BACK), after, "one press back");
        assert_eq!(steps(&game, BACK), card, "two presses back");
        assert_eq!(steps(&game, BACK), midboss, "three presses back");
        assert_eq!(steps(&game, BACK), start, "four presses back");

        // And the press after that, which has nowhere left to go inside the stage.
        game.press(BACK);
        assert!(
            log.said(&format!("step: held at the stage's start (script {start})")),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            start,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same three: what a step forward runs to is the next start where the
        // stage has one.
        assert_eq!(steps(&game, NEXT), midboss, "one press forward");
        assert_eq!(steps(&game, NEXT), card, "two presses forward");
        assert_eq!(steps(&game, NEXT), after, "three presses forward");
    });
}

/// **And a step forward does not leave the stage** — and what stops it over this game is the fight the
/// stage ends with being won.
///
/// **Which of the three walls a game gets is a reading of that game**, and this one's is measured: a launch
/// of the clear wrote **no `chain:` line at all** across the boundary between stage 1 and stage 2 — the
/// eleven jobs of a stage were the same either side of it — and **no `run: stage 0` line between them**, so
/// the player's job never leaves the chain and the stage word never reads nought. That rules out both of
/// the walls th06 and th06c give, a frame outside the gameplay scene and a frame whose stage word is
/// nought, and leaves th07's: `Chapters::stage_won`, which a pass over th07 needed after a `next` at the
/// end of a fight ran on and came to rest in stage 2 with the stage it was judging torn down behind it.
///
/// **And the fight is made the stage's own by its song**, which is how `Chapters::observe` tells one from a
/// midboss where the game answers nothing itself: `Th06nc` has no reading of whether the fight's own song
/// is playing, so what is left is the track against the stage's — and the four boundaries above are a
/// midboss's precisely because the laid-out stage plays no track at all.
#[test]
fn a_step_forward_inside_a_stage_of_th06nc_does_not_leave_it() {
    in_its_own_process(|| {
        let game = judging("th06nc-stepping-the-wall");
        let log = game.log();
        let (_, _, card, after) = the_boundaries(&game);

        // The fight this stage ends with, under a song of its own and far enough past the boundary before
        // it to be a chapter rather than folded into that one.
        game.frames(APART);
        game.image().plays_the_track(THE_FIGHTS_OWN_TRACK);
        game.image().a_boss_is_fighting(THE_FIGHTS_LIFE, 0);
        game.frames_until("the fight's own chapter", APART, || log.said("chapter 5"));
        let the_fight = game.state().stage_frames;
        // And won, which is the wall: the stage is over and every chapter of it is still a snapshot.
        game.image().no_boss_is_fighting();

        let held = log.written();
        game.press(NEXT);
        assert_eq!(
            game.state().stage,
            0,
            "the step forward left the stage it was made in:\n  {}",
            log.lines().split_off(held).join("\n  ")
        );
        assert!(
            log.said_since(held, "step: held in stage 1"),
            "the step forward came to rest nowhere:\n  {}",
            log.lines().split_off(held).join("\n  ")
        );
        // And the wall says which stage has ended and how to leave it.
        assert!(
            log.said_since(
                held,
                "step: stage 1 has ended; the across key with next or back leaves it"
            ),
            "the wall did not say which stage had ended:\n  {}",
            log.lines().split_off(held).join("\n  ")
        );
        // And the wall stands where the fight was won and no further: `Chapters::stage_won` is read a frame
        // later than it is set, so the frame held is the one after the win rather than the chapter's own.
        assert!(
            (the_fight..the_fight + APART).contains(&game.state().stage_frames),
            "the wall stood at frame {} where the fight's chapter began at {the_fight}",
            game.state().stage_frames,
        );
        // And back from that wall, which is what makes it a place to be rather than one a pass loses the
        // stage from: every chapter of the stage is still a snapshot orb holds. **Strictly behind where the
        // game is**, which is what puts the first press on the boundary before the fight rather than on the
        // one it is standing on.
        assert_eq!(steps(&game, BACK), after, "a step back from the wall");
        assert_eq!(steps(&game, BACK), card, "two presses back from the wall");
    });
}

/// **And the way out of a stage is one this game declines too**, which is what the across key finds: it says
/// so and the pass stays where it was.
///
/// `Th06nc::jump_to_stage` answers false, the second argument of this game's own replay start —
/// `0x14004ede0`, which the front end calls with the slot in `rcx` — not being read: the per-stage pointers
/// at `+0x40` and the null test either side of them are, and what the call wants beside the slot is not, so
/// a jump written from a guess is a playback asked for a stage nobody can say it would start. See the
/// change's own TODO and `th06nc/asm.md`'s *What is not established*.
#[test]
fn the_across_key_over_th06nc_says_the_replay_has_no_other_stage() {
    in_its_own_process(|| {
        let game = judging("th06nc-stepping-across");
        let log = game.log();
        let (_, _, card, after) = the_boundaries(&game);
        // A little way past the last boundary, so that a step back afterwards has that boundary to land on:
        // from the frame one began, a step back goes to the one before it.
        game.frames(60);
        game.press(HOLD);

        let from = log.written();
        game.keyboard().set(ACROSS, true);
        game.press(NEXT);
        game.keyboard().set(ACROSS, false);
        assert!(
            log.said_since(from, "step: the replay has no stage"),
            "the across key was taken as a move this game can make:\n  {}",
            log.lines().split_off(from).join("\n  ")
        );

        // And the stage is still the one the pass was in, with its own boundaries to step between.
        assert_eq!(steps(&game, BACK), after, "a step back after the refusal");
        assert_eq!(steps(&game, BACK), card, "two presses back after it");
    });
}
