//! **A replay played out the same way after being moved between its stages, to the last digit.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine. This is what `--collect` and `--judge` stand on: a pass that builds a midstage chapter table
//! steps between boundaries and moves between stages, so a replay that stops playing out the same way
//! makes every boundary it judged afterwards worthless.
//!
//! **How it was settled**: a line per update — the replay's input clock, the buttons it fed, the player's
//! position, and how many numbers the generator has given out — written for two passes over stage 1 and
//! compared. **9011 frames**, the whole stage including the boss, agreeing **to the last digit** across a
//! move out to stage 2 and back.
//!
//! What the laid-out game plays is a record of its own — `Fake::watches_a_replay_of_its_stages` — and the
//! comparison is the same one, field for field, over the frames a scenario has the patience for.

mod fake;

use fake::th06::{Fake, the_run};
use fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::RECORD_ENDS_AT;
use orb_core::game::{Game, Reproduction};

/// The two keys a stage move is made of, as orb fixes them: the across key held, and next or back
/// pressed. `orb_config::keys` is where their numbers are, orb's own names for them being its own.
const ACROSS: u8 = orb_config::keys::SHIFT.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;

/// How far into stage 1 these go before moving out of it.
///
/// Past the frames the panel is laid over a stage's start and well into the waves, so the stage being
/// left is a stage that was being played rather than one still loading.
const INTO_THE_STAGE: u32 = 260;

/// How many updates of the stage the two passes are compared over.
///
/// The real comparison was the whole of stage 1 — 9,011 frames, the boss included. This is what a
/// scenario has the patience for, and what it is enough of is the claim: the two passes agree from the
/// stage's first update, so a disagreement anywhere would be one this finds.
const COMPARED: u32 = 400;

/// A launch watching a replay of its own stages, with the stepping keys on: that is the only
/// configuration a stage move happens in, `--collect` being what it is for.
fn watching(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.chapter_stepping = true;
        config.during_replay = true;
        config.log_level = LogLevel::Verbose;
    });
    game.watches_a_replay_of_its_stages();
    // The frame stage 1 was built in, which is where a pass over it begins — see
    // [`moves_to_the_stage`]. The scene says a stage is playing from the frame it is asked for, so what
    // says the build has happened is the stage number the build itself raises.
    game.frames_until("stage 1 built", 8, || at_the_stage_built(&game, 0));
    game
}

/// Whether the game is on the frame `stage` was built in: the stage the build raised the number to, with
/// its own frame counter still at nothing.
fn at_the_stage_built(game: &Fake, stage: i32) -> bool {
    let run = game.state();
    run.playing && run.stage == stage && run.stage_frames == 0
}

/// A stage move: the across key held while next or back is pressed, and the game left on the frame that
/// stage was built in.
///
/// **On that frame and not one past it**, because that is where a pass over the stage begins: the build
/// leaves the stage's own frame counter at nothing and its jobs not yet updated, so a scenario that ran
/// one more frame would be comparing the stage's second update against another pass's first.
///
/// # Panics
/// Where the game does not arrive in a stage again, naming the one that was asked for.
fn moves_to_the_stage(game: &Fake, key: u8, stage: i32) {
    let log = game.log();
    game.keyboard().set(ACROSS, true);
    game.keyboard().set(key, true);
    game.frame();
    game.keyboard().set(key, false);
    game.keyboard().set(ACROSS, false);
    assert!(
        log.said(&format!(
            "stage {}: asked the game to start the replay there",
            stage + 1
        )),
        "the move to stage {} was not asked for:\n  {}",
        stage + 1,
        log.lines().join("\n  ")
    );
    game.frames_until(&format!("stage {} built", stage + 1), 60, || {
        at_the_stage_built(game, stage)
    });
}

/// The run's recording is not ended by a stage teardown, so leaving a stage part way does not blank it.
///
/// The game ends the run's recording at every stage teardown, writing a blank input and a frame number no
/// run reaches into the record it holds — which during playback is the replay's own, at the entry playback
/// has reached. Leaving a stage part way therefore terminated it there, and playing it again the player
/// took no input from that frame on.
///
/// Measured before this was held back, in the log at **297414375ms**: out of stage 1 around script frame
/// 250 and straight back, **three lives gone by frame 1027**.
#[test]
fn a_stage_teardown_does_not_end_the_recording_being_played_back() {
    in_its_own_process(|| {
        let game = watching("a-replay-the-teardown");
        // The record as it was loaded, which is what has to still be there afterwards.
        let recorded = game.image().recorded_inputs(0);
        assert!(
            recorded.len() > 2,
            "a record of {} entries is not one a teardown could be seen writing over",
            recorded.len(),
        );
        assert!(
            !recorded.iter().any(|(frame, _)| *frame == RECORD_ENDS_AT),
            "the record was terminated before any stage had been torn down",
        );

        // Out of the stage part way through it, which is a teardown: the game cuts the stage's own jobs
        // and ends the run's recording on the way.
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        moves_to_the_stage(&game, NEXT, 1);
        assert!(
            game.log()
                .said("replay: the record is being watched, not written; its terminator dropped"),
            "the teardown's write into the record was not held back:\n  {}",
            game.log().lines().join("\n  ")
        );

        // And the record is the one that was loaded, entry for entry: nothing of it was written over, so
        // the inputs past the frame the stage was left on are still there to be fed.
        assert_eq!(
            game.image().recorded_inputs(0),
            recorded,
            "the teardown wrote over the record being played back",
        );

        // Straight back, which is the move the measurement was taken on, and the stage plays on from its
        // own first frame with the inputs it always had.
        moves_to_the_stage(&game, BACK, 0);
        game.frames_until("the stage played again", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        assert_eq!(
            game.image().recorded_inputs(0),
            recorded,
            "the second teardown wrote over the record",
        );
        assert_eq!(
            game.state().deaths,
            0,
            "the player stood still and was hit, which is what a blanked record does to a replay",
        );
    });
}

/// Starting a replay at a stage puts the score and the extra lives paid for back to nothing first.
///
/// Starting at a stage is not the run carrying on. Measured before this: stage 1 began with the score the
/// run was left on — **7417420** at 303310937ms — crossed 10,000,000 at frame 3240 and took an extra life
/// the recording never had, which raised rank from **21 to 23**. The generator's stream shifted three
/// frames 173 frames later, and the player, moving on recorded inputs, was hit.
///
/// The extra lives go back to nothing rather than to what the score says, because the count only rises: a
/// stage's own loop can raise it from zero to what the restored score has paid for and cannot lower it
/// from what a later stage had reached.
#[test]
fn starting_a_replay_at_a_stage_puts_the_score_and_the_extra_lives_back_to_nothing() {
    in_its_own_process(|| {
        let game = watching("a-replay-the-score");
        // A run with a score behind it and the lives that score has paid for, which is what a replay left
        // part way through has: the stage being moved to is not the run carrying on.
        let scored = 7_417_420;
        game.image()
            .reproducing(orb_core::game::th06::image::Reproducing {
                score: scored,
                ..game.image().reproducing_now()
            });
        game.image().set_extra_lives(2);
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        assert!(
            game.image().reproducing_now().score >= scored,
            "the score this scenario is about did not survive to the move",
        );

        moves_to_the_stage(&game, NEXT, 1);
        assert_eq!(
            game.image().reproducing_now().score,
            0,
            "the stage began with the score the run was left on, which buys an extra life the \
             recording never had",
        );
        assert_eq!(
            game.image().extra_lives(),
            0,
            "the extra lives a later stage had paid for were carried into this one",
        );
    });
}

/// A screen shake does not outlive the stage that started it, because it draws from the generator.
///
/// A shake writes the play field's rectangle from **two numbers out of the generator every frame**, and
/// `Player::AddedCallback` measures where the player starts from that rectangle. Outliving the stage is
/// deliberate for the fade between two stages and wrong for a shake.
///
/// Measured: a bomb within the shake's **80 frames** of a stage move began the next stage with the player
/// at **`192.00,380.87`** where it starts one at **`192.00,384.00`**, and four numbers a frame going out
/// of the stream. Checked by leaving stage 2 while a bomb's shake was running — taken down at
/// **61041250ms** — and holding the stage 1 that followed against a menu-started pass of it: identical
/// from its first frame to the **742nd**, where the replay was stopped by hand.
#[test]
fn a_screen_shake_does_not_reach_the_stage_after_the_one_that_started_it() {
    in_its_own_process(|| {
        let game = watching("a-replay-the-shake");
        // Where a stage starts the player, which is what the next one has to start it at as well: measured
        // from the arcade region, and that region is what a shake writes over.
        let starts_at = unsafe { Th06.reproduction() }.player;
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // 被弾するより先にボム. Inside the shake's own 80 frames of the move, which is the case: a shake
        // that had run out would have put the region back itself.
        game.bombs();
        game.frames(SHAKE_FRAMES / 2);
        assert!(
            game.image().shaking_the_screen(),
            "the bomb's shake was not running at the move, so there was nothing for orb to take down",
        );
        // And it really has moved the region, which is what makes the assertion below say anything: a
        // shake that wrote nothing would leave the next stage's player where it belongs for no reason.
        let field = Th06.play_area();
        assert_ne!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the shake left the arcade region where the stage had it",
        );

        // The move, which takes the shake down and writes the region back where the shake would have left
        // it — the shake only does that on the frame it removes itself on, and it is being removed early.
        moves_to_the_stage(&game, NEXT, 1);
        assert!(
            game.log()
                .said("stage move: a screen shake was still running, and is taken down"),
            "the shake was carried into the stage after the one that started it:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            !game.image().shaking_the_screen(),
            "the shake's own job is still in the chain after the move",
        );
        assert_eq!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the arcade region was left where the shake had it",
        );

        // Which is what the next stage's player is measured from, so it starts where a stage starts one.
        assert_eq!(
            unsafe { Th06.reproduction() }.player,
            starts_at,
            "the stage after the shake began the player somewhere a stage does not",
        );
    });
}

/// And a shake left to run its own frames out puts the arcade region back itself, so a move after it has
/// nothing to take down.
///
/// **Which is what makes the scenario above say anything.** The region coming back there is orb writing
/// it, and nothing in a scenario that only ever cuts a shake early can tell that from a shake which would
/// have done it anyway. So this is the other half: the same bomb, the same stage move, and the shake's own
/// 80 frames allowed to finish in between.
#[test]
fn a_screen_shake_left_to_run_out_puts_the_region_back_itself() {
    in_its_own_process(|| {
        let game = watching("a-replay-the-shake-run-out");
        let starts_at = unsafe { Th06.reproduction() }.player;
        let field = Th06.play_area();
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // 被弾するより先にボム, and this time the whole of the shake: it moves the region while it runs,
        // and on its last frame it writes the region back and takes its own job out of the chain.
        game.bombs();
        game.frames(SHAKE_FRAMES / 2);
        assert_ne!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the shake left the arcade region where the stage had it",
        );
        game.frames(SHAKE_FRAMES / 2);
        assert!(
            !game.image().shaking_the_screen(),
            "the shake's own job is still in the chain after its {SHAKE_FRAMES} frames",
        );
        assert_eq!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the shake did not put the arcade region back on the frame it removed itself on",
        );

        // So the move has nothing to say about it, and the next stage starts its player where a stage
        // starts one for a reason of the game's own rather than one of orb's.
        moves_to_the_stage(&game, NEXT, 1);
        assert!(
            !game
                .log()
                .said("stage move: a screen shake was still running, and is taken down"),
            "orb took down a shake that had already taken itself down:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            unsafe { Th06.reproduction() }.player,
            starts_at,
            "the stage after the shake began the player somewhere a stage does not",
        );
    });
}

/// How many frames of a shake this file waits out before the move, which has to be inside the 80 a shake
/// runs for: a shake that had finished would have put the arcade region back itself.
///
/// 紅魔郷's own **80**, which is also the whole of what a shake left to run out takes — see
/// [`a_screen_shake_left_to_run_out_puts_the_region_back_itself`].
const SHAKE_FRAMES: u32 = 80;

/// The whole of it: two passes over one stage agreeing to the last digit across a move and back.
///
/// **9011 frames**, the whole stage including the boss, with the replay's input clock, the buttons it fed,
/// the player's position and the count of numbers the generator has given out identical on both sides.
/// This is the scenario the ones above are the parts of, and the one `--collect` and `--judge` rest on.
#[test]
fn a_stage_played_twice_across_a_move_agrees_to_the_last_digit() {
    in_its_own_process(|| {
        let game = watching("a-replay-twice");
        let pass = |over: u32| {
            assert_eq!(
                game.state().stage_frames,
                0,
                "a pass began somewhere other than the frame the stage was built in",
            );
            let mut line = Vec::with_capacity(over as usize);
            for _ in 0..over {
                game.frame();
                line.push(unsafe { Th06.reproduction() });
            }
            line
        };

        // The stage straight through, from its first update.
        let first = pass(COMPARED);

        // And out to stage 2 and back, which is what a judging pass does between boundaries: the stage is
        // torn down, the next one built, and this one built again from its own beginning.
        moves_to_the_stage(&game, NEXT, 1);
        moves_to_the_stage(&game, BACK, 0);
        let second = pass(COMPARED);

        // To the last digit, and the first update they differ on is the whole of the diagnosis: what is
        // wrong is whichever of the fields moved, and how far in says what moved it.
        let differ = first
            .iter()
            .zip(&second)
            .position(|(before, after)| before != after);
        assert_eq!(
            differ.map(|at| (at, first[at], second[at])),
            None,
            "the two passes came out of step",
        );
        // Said out loud, because a comparison of two empty lines passes: the pass has to have moved the
        // player and drawn from the generator, or there is nothing in it to agree about.
        let moved: Vec<&Reproduction> = first
            .iter()
            .filter(|line| line.player != first[0].player)
            .collect();
        assert!(
            !moved.is_empty(),
            "the recorded inputs never moved the player, so the two passes agree about nothing",
        );
        assert!(
            first.last().expect("an update").randoms > first[0].randoms,
            "the stage drew nothing from the generator over {COMPARED} updates",
        );
    });
}
