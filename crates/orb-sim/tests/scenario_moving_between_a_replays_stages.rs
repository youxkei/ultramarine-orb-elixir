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
use orb_core::game::th06::image::{RECORD_ENDS_AT, chain_job};
use orb_core::game::{Game, Reproduction};

/// The two keys a stage move is made of, as orb fixes them: the across key held, and next or back
/// pressed. `orb_config::keys` is where their numbers are, orb's own names for them being its own.
const ACROSS: u8 = orb_config::keys::SHIFT.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;

/// What the first extra life a run's score pays for costs, which is 紅魔郷's own **10,000,000**:
/// `g_ExtraLivesScores`' first entry, and the threshold `GameManager::OnUpdate`'s own `while` loop raises
/// `extraLives` past.
const AN_EXTRA_LIFE: u32 = 10_000_000;

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
/// its own frame counter on the **first** of its updates.
///
/// One and not nothing, because a scene's first update is on the frame it was built — the supervisor is the
/// calc chain's first job and everything it registers goes in behind the walk's own position, so the walk
/// reaches the new stage before it returns. See `scenario_the_frame_a_scene_is_built_on.rs`.
fn at_the_stage_built(game: &Fake, stage: i32) -> bool {
    let run = game.state();
    run.playing && run.stage == stage && run.stage_frames == 1
}

/// A stage move: the across key held while next or back is pressed, and the game left on the frame that
/// stage was built in.
///
/// **On that frame and not one past it**, because that is where a pass over the stage begins: the build
/// frame is the stage's own first update, so a scenario that ran one more frame would be comparing the
/// stage's second update against another pass's first.
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
        // Below the first extra life, rather than nothing: the frame the stage was built in is a frame the
        // stage has been updated on, so what is read here is one update's own scoring. What the claim is
        // about is that the run's score did not survive into it — nothing this stage can have scored in one
        // update comes near [`AN_EXTRA_LIFE`], and the score it was left on is past it.
        assert!(
            game.image().reproducing_now().score < AN_EXTRA_LIFE,
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

/// A shake's own frames each draw **four** numbers out of the generator, which is what makes one left running
/// across a stage move take the stream with it.
///
/// `ScreenEffect::ShakeScreen` calls `g_Rng.GetRandomU32InRange(3)` once per axis, and `GetRandomU32` is two
/// `GetRandomU16`s — each of which raises `generationCount`. So two per axis and four a frame, which is the
/// number the measurement above recorded off the running game.
///
/// The frame a shake removes itself on draws none: `timer >= effectLength` returns before the offset is worked
/// out, so that one puts the arcade region back and nothing else. Which is why this counts over frames well
/// inside the shake's own [`SHAKE_FRAMES`] rather than over the whole of it.
///
/// Against a span of the same stage with no shake in it, because a stage draws from the generator itself: what
/// the claim is about is the four the shake adds, not what a frame comes to.
#[test]
fn a_shakes_own_frames_each_draw_four_numbers() {
    in_its_own_process(|| {
        let game = watching("a-replay-the-shakes-draws");
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // The stage on its own first.
        let before = unsafe { Th06.reproduction() }.randoms;
        game.frames(COUNTED);
        let quiet = unsafe { Th06.reproduction() }.randoms - before;

        // And the same span with a shake running through the whole of it.
        game.bombs();
        let at_the_bomb = unsafe { Th06.reproduction() }.randoms;
        game.frames(COUNTED);
        assert!(
            game.image().shaking_the_screen(),
            "the shake was over before the span this counts, so what it counted is not a shake's",
        );
        let shaken = unsafe { Th06.reproduction() }.randoms - at_the_bomb;
        assert_eq!(
            shaken - quiet,
            4 * COUNTED,
            "a shake of {COUNTED} frame(s) drew {} numbers where the stage alone drew {quiet}",
            shaken,
        );
    });
}

/// How many frames of a shake the count above runs over, which has to be inside the ones a shake runs for and
/// far enough that a per-frame count is not one frame's rounding.
const COUNTED: u32 = 40;

/// A job that answers `CONTINUE_AND_REMOVE_JOB` is cut **by the walk**, and the walk goes on.
///
/// `ScreenEffect::ShakeScreen` is the one job of this game's chain that ever asks: on the frame its own
/// frames run out it puts the arcade region back and returns that answer, and `Chain::RunCalcChain`'s switch
/// is what reads the next element, calls `Cut` on the one that answered, and carries on from there.
///
/// **What no scenario could ask before the walk existed** — see
/// [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md), where this
/// is one of the three things the last step of it makes reachable. The job cutting itself and the walk
/// cutting it leave the same memory behind, so what this is the record of is that the answer is what does it:
/// the job calls no `Chain::Cut` at all, and the element is gone all the same.
#[test]
fn a_job_that_asks_to_be_removed_is_cut_by_the_walk_and_the_walk_goes_on() {
    in_its_own_process(|| {
        let game = watching("a-replay-a-job-removed");
        let field = Th06.play_area();
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        let without_a_shake = game.image().calc_chain_jobs();

        // ボム, which is one more job in the chain.
        game.bombs();
        assert_eq!(
            game.image().calc_chain_jobs(),
            [without_a_shake.clone(), vec![chain_job::SCREEN_EFFECT]].concat(),
            "the bomb's job did not go into the chain behind the ones already in it",
        );

        // Every frame of it but the last, which is the one it asks to be removed on.
        game.frames(SHAKE_FRAMES - 1);
        assert!(
            game.image().shaking_the_screen(),
            "the shake asked to be removed before its own last frame",
        );
        let played = game.state().stage_frames;

        // And that last frame.
        game.frame();
        assert_eq!(
            game.image().calc_chain_jobs(),
            without_a_shake,
            "the walk did not take out the job that asked to be removed, or took out another with it",
        );
        // The job ran on the frame it asked on, which is what `CONTINUE_AND_REMOVE_JOB` is and not
        // `BREAK`: the region is back where the shake found it.
        assert_eq!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the job that asked to be removed did not run on the frame it asked on",
        );
        // And the walk went on rather than ending: the stage was played on that frame and on the next.
        assert_eq!(
            game.state().stage_frames,
            played + 1,
            "the frame a job asked to be removed on is a frame the stage was not played on",
        );
        game.frame();
        assert_eq!(
            game.state().stage_frames,
            played + 2,
            "the game stopped being played after a job asked to be removed",
        );
    });
}

/// And `Chain::Cut` on a job in the middle of the list takes that one out and no other.
///
/// Which is what orb does at a stage move: a shake still running is taken down through the game's own
/// `Chain::Cut` — see [`a_screen_shake_does_not_reach_the_stage_after_the_one_that_started_it`], which is
/// what that cut is *for*. This is the other half of it, and the half a list has: the shake sits at
/// `TH_CHAIN_PRIO_CALC_SCREENEFFECT`, which is 14, with the supervisor's own job at 0 and the gameplay
/// scene's at 4 in front of it — so cutting it is a relink of the element before it and nothing else.
///
/// **Also not askable before the walk**: until the jobs were a list in the game's own memory there was
/// nothing for a cut to be in the middle of.
#[test]
fn a_cut_in_the_middle_of_the_chain_takes_out_that_job_and_no_other() {
    in_its_own_process(|| {
        let game = watching("a-replay-a-cut-in-the-middle");
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        game.bombs();
        game.frames(SHAKE_FRAMES / 2);
        // The list as it stands: the supervisor, the scene, and the shake behind them both.
        assert_eq!(
            game.image().calc_chain_jobs(),
            vec![
                chain_job::SUPERVISOR,
                chain_job::GAMEPLAY,
                chain_job::SCREEN_EFFECT
            ],
            "the chain is not the three jobs this is about cutting one of",
        );

        // The move, which is where orb calls the game's own `Chain::Cut` on the shake.
        moves_to_the_stage(&game, NEXT, 1);
        assert!(
            game.log()
                .said("stage move: a screen shake was still running, and is taken down"),
            "orb did not cut the shake, so nothing was cut at all:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.image().calc_chain_jobs(),
            vec![chain_job::SUPERVISOR, chain_job::GAMEPLAY],
            "the cut took out a job beside the one it was asked for, or left that one in",
        );
    });
}

/// How many frames of a shake this file waits out before the move, which has to be inside the ones a shake
/// runs for: a shake that had finished would have put the arcade region back itself.
///
/// The game's own count, out of the fake rather than written again here, so that the number a shake runs for
/// and the number this waits out cannot drift apart — see [`fake::th06::SHAKE_FRAMES`], which is also the
/// whole of what a shake left to run out takes.
const SHAKE_FRAMES: u32 = fake::th06::SHAKE_FRAMES as u32;

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
                1,
                "a pass began somewhere other than the frame the stage was built in",
            );
            // That frame's own numbers first, it being the stage's first update: a pass that only recorded
            // what the frames after it left would be a pass with the first update missing from both sides.
            let mut line = Vec::with_capacity(over as usize + 1);
            line.push(unsafe { Th06.reproduction() });
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
