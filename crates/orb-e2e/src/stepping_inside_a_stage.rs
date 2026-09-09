//! **The stepping keys on their own: one chapter at a time, inside the stage.**
//!
//! `moving_between_a_replays_stages.rs` is the other half of the same hand — the across key held, which
//! leaves the stage altogether — and `a_chapter_table_collected.rs` is the dropped key, which aims at a
//! boundary judged out of the table. What is left is the press with nothing held, which is the one a
//! judging pass spends its afternoon on: back onto the boundary just passed, and forward onto the next.
//!
//! Both go through `Chapters::previous_start` and `next_start`, whose whole job is that a step lands
//! **strictly** either side of where the game is — so a press moves rather than staying put, and a frame or
//! two past a boundary steps back onto it.
//!
//! And the end of the stage is a wall. The game leaving the gameplay scene to build the next one is well
//! after the boss went down, and nothing has been torn down yet: the hold is what stops the update that
//! would do it. So there the hold key and a step forward do not move, and **a step back still goes where it
//! always goes** — which is what makes the wall a place to be rather than a place a pass loses the stage
//! from.

use crate::fake::th06::{ATTACK_CHANGES, BOSS_ARRIVES, CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Playing, Scene};

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;
const ACROSS: u8 = orb_config::keys::SHIFT.0;
const DROP: u8 = orb_config::keys::DOWN.0;

/// The stage frame this stage's own start is a chapter at.
///
/// `STAGE_SETTLE_FRAMES` past the frame the stage was noticed on, which is its first update, with the
/// music already streaming so that the wait for it is over the moment it is asked about. A stage with no
/// song spends the whole of `MUSIC_WAIT_FRAMES` on top instead and takes its first chapter 240 frames
/// later — far enough into the gap in the waves that the shortest a chapter may be would swallow the
/// boundary the last e2e test here is about, which is why every one of them streams its song.
const THE_STAGES_OWN_START: u32 = 9;

/// Where a pass over a replay of its own stages begins: the stepping keys on, the song streaming, and the
/// game on the frame stage 1 was built in.
fn stepping(name: &str, tuning: bool) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.chapter_stepping = true;
        config.during_replay = true;
        config.chapter_tuning = tuning;
        config.log_level = LogLevel::Verbose;
    });
    game.streams_its_song(0);
    game.watches_a_replay_of_its_stages();
    game.frames_until("stage 1 built", 8, || {
        let run = game.state();
        run.playing && run.stage == 0 && run.stage_frames == 1
    });
    game
}

/// One press of a stepping key with nothing else held, and the frame it left the game on.
///
/// # Panics
/// Where the step wrote no line saying where it held the game, which is a press that did nothing.
fn steps(game: &Fake, key: u8) -> u32 {
    let held = game.log().lines().len();
    game.press(key);
    let said = game.log().lines().split_off(held).join("\n  ");
    assert!(
        said.contains("step: held"),
        "the press of {key:#04x} held the game nowhere:\n  {said}",
    );
    game.state().stage_frames
}

/// Back onto each chapter start in turn, and forward again.
///
/// The chapters of this stage are its own start and the three the fight has — the boss arriving, the card
/// it puts up, and the attack after that card — with nothing of the baked table among them, its first
/// boundary for stage 1 being at script 4472. So the frames a step lands on are the fight's own, and every
/// one of them is a number this game declares rather than one written here.
#[test]
fn a_step_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = stepping("stepping-inside", false);
        let log = game.log();

        // Past the last of the fight's chapters, and held a little way into it: a step back from inside a
        // chapter goes to that chapter's own start, which is what makes a press the way onto the boundary
        // just passed.
        let past = ATTACK_CHANGES + 100;
        game.frames_until("the stage played past its fight", past + 60, || {
            game.state().stage_frames >= past
        });
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the third press does not stay on the
        // frame the second landed on.
        assert_eq!(steps(&game, BACK), ATTACK_CHANGES, "one press back");
        assert_eq!(steps(&game, BACK), CARD_STARTS, "two presses back");
        assert_eq!(steps(&game, BACK), BOSS_ARRIVES, "three presses back");
        assert_eq!(
            steps(&game, BACK),
            THE_STAGES_OWN_START,
            "four presses back",
        );

        // And the press after that, which has nowhere left to go inside the stage: the stage's own start is
        // what lies before its first boundary, and the restore has already arrived there.
        game.press(BACK);
        assert!(
            log.said(&format!(
                "step: held at the stage's start (script {THE_STAGES_OWN_START})"
            )),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            THE_STAGES_OWN_START,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same three: what a step forward runs to is the next start where the
        // stage has one, and the chapter it was in changing where it has not.
        assert_eq!(steps(&game, NEXT), BOSS_ARRIVES, "one press forward");
        assert_eq!(steps(&game, NEXT), CARD_STARTS, "two presses forward");
        assert_eq!(steps(&game, NEXT), ATTACK_CHANGES, "three presses forward");
    });
}

/// How long this game's stage runs before the next one begins, for the e2e test about its end.
///
/// Past the fight, so that the stage being walled is one that was played rather than one still loading.
const STAGE_ENDS: u32 = ATTACK_CHANGES + 60;

/// The end of the stage is a wall: held there and kept held, with a step back still going where it always
/// goes.
#[test]
fn a_stage_that_has_ended_is_held_there_until_the_across_key_leaves_it() {
    in_its_own_process(|| {
        let game = stepping("stepping-the-wall", false);
        let log = game.log();
        game.stages_last(STAGE_ENDS);

        // The game leaving the gameplay scene to build the next stage, which is what the end of a stage is
        // from outside: the scene it is rebuilding its manager in, with the run still the same run.
        game.frames_until("the stage's end", STAGE_ENDS + 60, || !game.state().playing);
        assert_eq!(
            game.image().scene(),
            Scene::Rebuilding,
            "the stage did not end in the scene the next one is built in",
        );
        // And one frame more, that being where the step reads it: what a frame's stepping is decided on is
        // the state the frame before it left, so the end of the stage is met on the frame after the one it
        // happened on.
        game.frame();
        assert!(
            log.said("step: stage 1 has ended; the across key with next or back leaves it"),
            "the wall was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );

        // And it is kept there. The hold key is a toggle everywhere else in the stage and moves nothing
        // here — carrying on is what tears the stage down — and neither does a step forward.
        let at_the_wall = game.state();
        game.press(HOLD);
        game.press(NEXT);
        game.frames(30);
        assert_eq!(
            game.state(),
            at_the_wall,
            "the game was let past the end of the stage",
        );
        assert_eq!(
            game.image().scene(),
            Scene::Rebuilding,
            "the stage was torn down at its own end",
        );

        // A step back, though, goes where it always goes: the stage's start comes back, and with it the
        // scene the stage is played in.
        assert_eq!(
            steps(&game, BACK),
            ATTACK_CHANGES,
            "a step back from the wall",
        );
        assert!(game.state().playing, "the stage did not come back");

        // And the across key is what leaves: the stage after this one, asked for by the game.
        game.press(HOLD);
        game.frames_until("the wall again", STAGE_ENDS + 60, || !game.state().playing);
        game.frame();
        game.keyboard().set(ACROSS, true);
        game.press(NEXT);
        game.keyboard().set(ACROSS, false);
        assert!(
            log.said("stage 2: asked the game to start the replay there"),
            "the across key did not leave the stage that had ended:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// The script frame this game's first stage has the detector propose, out of `a_chapter_table_collected`'s
/// own reckoning: the gap in its waves begins at script 200 and a proposal goes `ENEMY_GAP_FRAMES` into
/// one.
const PROPOSED_AT: u32 = 259;

/// **A boundary judged out of the table is not a place a step stops at**, though the chapter it began is
/// still in the stage's list of starts.
///
/// Which is the one thing `Chapters::kept` decides. Judged out is not gone — the dropped key with a
/// stepping key is how it is reached, and taking a refusal back means being able to get to it — but it
/// begins no chapter any more, and a step that stopped there would be stopping at nothing.
#[test]
fn a_boundary_judged_out_of_the_table_is_stepped_over() {
    in_its_own_process(|| {
        let game = stepping("stepping-a-boundary-out", true);
        let log = game.log();

        // The proposal, and the chapter it begins — played past, so that a step back has somewhere to come
        // from.
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!("stage 1 chapter 2 at frame {PROPOSED_AT}"))
        });
        game.frames(30);
        game.press(HOLD);

        // A step back from inside its chapter lands on it, which is what makes the assertion below say
        // something: the frame is a place the stepping stops at until it is judged out. And landing on it is
        // also the one place the judging keys act, which is where the next press goes.
        assert_eq!(
            steps(&game, BACK),
            PROPOSED_AT,
            "the boundary was not a place a step stopped at while the table held it",
        );

        // Out of the table, which for the detector's own is `DROP` twice.
        game.press(DROP);
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {PROPOSED_AT} ADJUST -> DROP")),
            "the boundary was not judged out:\n  {}",
            log.lines().join("\n  ")
        );

        // And now a step back from past it goes to the stage's own start instead, the frame it is on being
        // in no table to be found in.
        game.press(HOLD);
        let into_the_fight = BOSS_ARRIVES + 60;
        game.frames_until(
            "the stage played into its fight",
            into_the_fight + 300,
            || game.state().stage_frames >= into_the_fight,
        );
        game.press(HOLD);
        assert_eq!(
            steps(&game, BACK),
            BOSS_ARRIVES,
            "a step back from inside the fight",
        );
        assert_eq!(
            steps(&game, BACK),
            THE_STAGES_OWN_START,
            "a step back over the boundary that was judged out",
        );
    });
}

/// Where the stop a run forward used to have fell, and a frame of the stage past it: ten minutes of game
/// time, written down as longer than any stage.
const PAST_THE_OLD_STOP: u32 = 60 * 60 * 10 + 100;

/// The script frame the detector proposes a boundary at past that: this game's waves are two hundred frames
/// on and two hundred off from the stage's start, so the gap the jump lands in front of has its proposal
/// where [`PROPOSED_AT`] sits in the stage's own first four hundred.
const PROPOSED_PAST_THE_STOP: u32 = 60 * 60 * 10 + PROPOSED_AT;

/// **A step back over a stage longer than ten minutes lands on the boundary and not on the stop that bounds
/// the run forward.**
///
/// A step back restores the stage's start and runs forward to the frame asked for, and that run has a stop on
/// it for a target it can never reach. The stop was ten minutes of game time, written down as longer than any
/// stage — and th07's Extra is not: a replay of it fought its boss to frame 42127, so every step back to a
/// chapter past the ten minutes ran out the stop and held the game wherever that fell, which read as a step
/// that worked. The stop is the stepping's own and not either game's, so the same step is asked of this game:
/// a chapter proposed past the ten minutes, and a step back onto it.
///
/// The stage is not played to that frame — the clocks are moved there, which is the one thing a laid-out
/// game can do that a real one cannot: what a step back replays from the stage's start is the buttons, and
/// what it has to reach is the frame.
#[test]
fn a_step_back_over_a_stage_longer_than_the_stop_lands_on_the_chapter_it_asks_for() {
    in_its_own_process(|| {
        // The table being built as well as stepped through, because of what the boundary has to be: with the
        // fight this game arranges for frame 400 jumped over, what a chapter past the ten minutes can begin at
        // is a gap in the waves, and the detector proposes one only while it is tuning.
        let game = stepping("stepping-a-long-stage", true);
        let log = game.log();

        // Past the stop the run forward used to have, without playing the frames: both of the stage's clocks,
        // the boundary being judged on the script's and the landing read off the stage's.
        let run = game.image().playing_now();
        game.image().playing(Playing {
            frames: PAST_THE_OLD_STOP,
            script_frames: PAST_THE_OLD_STOP as i32,
            ..run
        });
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!(
                "stage 1 chapter 2 at frame {PROPOSED_PAST_THE_STOP}"
            ))
        });
        // And the game past the old stop with it, which is what makes the step below one that stop would
        // have cut short.
        assert!(
            game.state().stage_frames > 60 * 60 * 10,
            "the game is inside the old stop, so this proves nothing: {}",
            game.state().stage_frames,
        );
        game.frames(30);
        game.press(HOLD);

        assert_eq!(
            steps(&game, BACK),
            PROPOSED_PAST_THE_STOP,
            "the step back stopped short of the chapter it asked for:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
