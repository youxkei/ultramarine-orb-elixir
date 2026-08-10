//! **`--clear`: a run to the ending in a minute, on nothing but the shot key held.**
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well, so it is also
//! how the ending, the replay screen and the score file's write on the way out were all reached. It
//! fixes the mode rather than asking — `mode: pointdevice to start with; nobody is asked`.
//!
//! **What makes these say anything is a bullet that can really kill.** `Fake::puts_a_bullet_on_the_player`
//! leaves one there and the hit test runs where the game runs it — after `Player::OnUpdate`, priority 7
//! against the bullets' 11 — so a run that comes out with `deaths=0` came out that way because something
//! stopped it, and the same bullet in a run without `--clear` kills on the frame it is put there.
//!
//! What these are about is how the run *went*, not how long it took: the wall clock is one machine's and is
//! not written down. The invariants are no death anywhere, the six stages in order, the ending after them,
//! and the screen that saves a replay never coming up.

use crate::fake::th06::{Fake, STAGES, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, result_state};

/// How long each stage of these runs is, in frames.
///
/// Past the 248 a stage spends settling before its first chapter, so every stage of the run is a stage
/// that took one — a stage nothing was written down in would be a stage this says nothing about.
const STAGE_FRAMES: u32 = 300;

/// How long a whole run of six of those needs, in frames, with room for the transitions and the ending.
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// The two names an open of the score file can land in: the one the game asks for, and orb's own beside it.
/// Which one each open lands in is `the_score_file.rs`'s subject; what these two are for is that
/// **neither** is written by a run nothing could hit.
const THEIRS: &str = "score.dat";
const OURS: &str = "pointdevice_score.dat";

/// A launch of `--clear`, in a run, with a bullet sitting on the player.
///
/// Two of the four things that word sets — `fast_clear` and the refusal of a replay write — and not the other
/// two, deliberately. `speed` stays 1 where `--clear` names 64: what these e2e tests count is frames of the
/// run, and a launch running 64 updates in each of them counts them in sixty-fourths. `resume` stays on
/// because nothing here writes a chapter down to be offered one.
fn clearing(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.fast_clear = true;
        config.block_replay_save = true;
        config.log_level = LogLevel::Verbose;
    });
    game.stages_last(STAGE_FRAMES);
    game.in_a_run_nobody_was_asked_about();
    game.puts_a_bullet_on_the_player();
    game
}

/// Six stages with nothing able to hit the player, and no death anywhere in them.
///
/// What `--clear` is for is a result screen without half an hour of playing well: stages 1 to 6 on nothing
/// but the shot key held, with `deaths=0` from the first stage to the ending and not one `died in chapter`
/// line. The invariants are what this asserts and not how long it took, which is one machine's.
#[test]
fn a_cleared_run_reaches_the_ending_with_no_death_in_it() {
    in_its_own_process(|| {
        let game = clearing("a-clear-through-six");
        // Nobody was asked which mode, which is what `--clear` is: it takes the mode it is given.
        assert!(
            game.log()
                .said("mode: pointdevice to start with; nobody is asked"),
            "the launch asked which mode:\n  {}",
            game.log().lines().join("\n  ")
        );

        // Every stage of the run, in order, each one a stage that took a chapter of its own.
        for stage in 1..=STAGES {
            game.frames_until(&format!("stage {stage}"), A_WHOLE_RUN, || {
                game.log()
                    .said(&format!("stage {stage} chapter 1 (stage start)"))
            });
            assert_eq!(
                game.state().deaths,
                0,
                "a death was counted before stage {stage} of a run nothing could hit",
            );
        }
        // And then the ending rather than a seventh stage, which is what clearing stage six is. Run out
        // inside the frame it began on rather than played, which is what `--clear` does with one: the
        // scene is never a scene anybody sees, so what says the run went through it is the line orb
        // writes about running it out. How many updates that takes and where the staff roll begins is
        // `the_ending.rs`'s.
        game.frames_until("the result screen", A_WHOLE_RUN, || {
            game.image().scene() == Scene::Result
        });
        assert!(
            game.log()
                .lines()
                .iter()
                .any(|line| line.contains("ending skipped") && line.contains("scene 10 -> 7")),
            "the run did not go through the ending on its way to the result screen:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.state().deaths,
            0,
            "a death was counted in a run nothing could hit",
        );
        // Not one, which is the other half: `deaths` is the game's count and this is orb's, and a run that
        // lost the frames of invulnerability would have both.
        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("died in chapter")),
            "orb noticed a death in a run nothing could hit:\n  {}",
            game.log()
                .lines()
                .iter()
                .filter(|line| line.contains("died in chapter"))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    });
}

/// And the write itself is refused, which is the other half of no replay being offered.
///
/// `--clear` is the one thing that sets `block_replay_save`, and the reason is in `orb_config`: a run whose
/// player nothing could hit leaves a record that plays back as a run which dies where this one did not. So the
/// screen being skipped is not enough — the write is refused as well, and `ReplayManager::SaveReplay`'s
/// non-null-path half is where that happens. Its null-path half is the teardown every way out of a run goes
/// through and is called through untouched.
///
/// **Both halves are here, because the first says nothing alone**: a hook that dropped every save would pass
/// it, and a launch nobody asked to block one has to write. Which is also where the gate is — a real launch
/// decides by not installing the hook, and a game that hands the function over has one call site whichever way
/// the launch was configured, so orb keeps the decision beside the hook instead.
#[test]
fn a_cleared_runs_replay_is_refused_and_an_ordinary_launchs_is_written() {
    in_its_own_process(|| {
        // ── The cleared run, whose save is refused.
        {
            let game = clearing("a-clear-the-replay-refused");
            game.saves_its_replay();
            assert!(
                game.replays_written().is_empty(),
                "a run nothing could hit wrote {:?}",
                game.replays_written(),
            );
            assert!(
                game.log().said("replay save blocked"),
                "orb did not say it refused the write:\n  {}",
                game.log().lines().join("\n  ")
            );
        }

        // ── And an ordinary launch, whose save goes through: the same call, the same game, and nothing of
        // orb's in the way of it.
        {
            let game = Fake::attach("a-clear-the-replay-written", the_run(), |config| {
                config.log_level = LogLevel::Verbose;
            });
            game.saves_its_replay();
            assert_eq!(
                game.replays_written().len(),
                1,
                "a launch nobody asked to block a save wrote {:?}",
                game.replays_written(),
            );
            assert!(
                !game.log().said("replay save blocked"),
                "orb refused a write nobody asked it to:\n  {}",
                game.log().lines().join("\n  ")
            );
        }
    });
}

/// The frames of invulnerability go back with the player's state, and this is why they have to.
///
/// With the frames left where the last respawn had put them, the player died in the chapter that had just
/// begun and again after every retry: `Player::OnUpdate` runs at chain priority **7** and the bullets are
/// checked at **11**, so the state expired before the hit test in the update it was written for. Writing the
/// frames left under it is what fixed that.
#[test]
fn the_invulnerability_outlasts_the_hit_test_in_the_update_it_was_written_for() {
    in_its_own_process(|| {
        // The same bullet in a run without `--clear`, first, because a bullet nothing dies to says
        // nothing about what stopped this run dying: it kills on the frame it is put there.
        {
            let game = Fake::attach("a-clear-a-lethal-bullet", the_run(), |config| {
                config.log_level = LogLevel::Verbose;
            });
            game.in_a_pointdevice_run();
            game.puts_a_bullet_on_the_player();
            game.frames_until("the death that bullet is", 8, || {
                game.log().said("died in chapter")
            });
        }

        // And with `--clear`, the same bullet against a player orb makes invulnerable before every update:
        // both the state and the frames left under it, so that `Player::OnUpdate` at priority 7 does not
        // run them out before the bullets are checked at 11.
        let game = clearing("a-clear-the-invulnerability");
        // One update of the stage, which is where orb writes it: `make_invulnerable` runs before the calc
        // chain and only over a run that is in progress, so a stage's own build frame has nothing yet.
        game.frame();
        let frames = game.image().invulnerable_frames();
        assert!(
            frames > 0,
            "orb wrote a player invulnerable with no frames left under it, which is a player the hit \
             test in this very update can kill",
        );

        // A whole stage of it, so the claim is not about one update: every one of those runs the hit test
        // against a live bullet, and none of them killed.
        game.frames_until("a stage of it", A_WHOLE_RUN, || {
            game.state().stage_frames > STAGE_FRAMES / 2
        });
        assert_eq!(game.state().deaths, 0);
        assert_eq!(game.state().lives, 2, "a life went to a bullet");
        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("died in chapter")),
            "the invulnerability ran out inside the update it was written for:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// No replay is offered, and the screen that saves one is skipped rather than the write refused.
///
/// The state is written after the screens ahead of the question and not instead of them — the high-score
/// name entry and the stats screen are played through as they always were, and
/// `the_screen_a_finished_run_ends_at.rs` is where that is asserted. What is asserted here is the write
/// itself and that no frame of the question was drawn.
#[test]
fn a_run_with_chapters_is_not_offered_the_screen_that_saves_a_replay() {
    in_its_own_process(|| {
        let game = clearing("a-clear-no-replay-offered");
        // The screen itself, which registers its own job on the frame after the scene: orb finds it by
        // that job's callback and by nothing else, there being nothing in the game's static data that
        // points at it.
        game.frames_until("the screen that saves a replay", A_WHOLE_RUN, || {
            game.log()
                .said("result: no replay is offered for a run with chapters")
        });
        assert!(
            game.log()
                .said("result: no replay is offered for a run with chapters"),
            "the screen that saves a replay was left to come up:\n  {}",
            game.log().lines().join("\n  ")
        );

        // Skipped rather than answered: what orb writes is the state the game itself puts a practice run's
        // result screen into, because answering the question means writing the interrupt each of its 38
        // sprites is to run next and waiting out the fade they play.
        assert_eq!(
            game.image().result_screen_state(),
            result_state::EXIT,
            "the question was answered rather than written past",
        );
        // And written before the frame the question would be drawn from, so no part of that screen ever
        // reached anybody: a state written a frame late is a question somebody has to answer.
        game.frames_until("the title menu after it", A_WHOLE_RUN, || {
            game.image().scene() == Scene::FrontEnd
        });
        assert!(
            !game.the_replay_question_was_drawn(),
            "the screen that saves a replay was drawn before orb wrote past it",
        );

        // And it was a finished run that reached it, which is what the write is only ever about: the
        // screens before the question are the subject of the e2e test named above.
        assert!(
            game.log().said("run ended after"),
            "the run did not finish through the result screen:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// Nothing is written to any score file, and the game's own teardown still runs.
///
/// The read goes through and the write is refused, and the game carries on past the refusal rather than
/// stopping on it: `WriteDataToFile` checks its open, returns -1, and its one caller — the `call` at
/// 0x42bc1a — drops that and frees the buffer either way. So both files come out byte for byte as they went
/// in.
#[test]
fn a_cleared_run_writes_no_score_and_leaves_the_games_teardown_in_the_path() {
    in_its_own_process(|| {
        let game = clearing("a-clear-writes-no-score");
        assert!(
            game.log().said("score: no file is written this run"),
            "the launch did not say it writes no file:\n  {}",
            game.log().lines().join("\n  ")
        );
        // What the game's own file holds going in, which is what it has to hold coming out.
        let theirs = game.score_file(THEIRS).expect("the game's own score file");

        // Reads are left alone, so the session still shows the ranking it had: the file was read on the way
        // into every stage of the run.
        game.frames_until("a read of the score file", A_WHOLE_RUN, || {
            game.score_file_opens().iter().any(|open| !open.write)
        });

        // And the write is refused where the run ends, which orb reaches by taking it through the ranking:
        // the file that must not be written is whichever *this* run would have written, so the refusal is
        // before the fork and covers both names.
        game.frames_until("the write refused", A_WHOLE_RUN, || {
            game.log()
                .said("score: nothing written, this run had nothing able to hit the player")
        });
        // Refused rather than sent somewhere else: the game asks for the file once, checks its open and
        // drops what its write returned, so a refusal is a file that stays as it was and a game that carries
        // on as if it had saved.
        assert_eq!(
            game.score_file(THEIRS).as_deref(),
            Some(theirs.as_slice()),
            "the game's own file was written by a run nothing could hit",
        );
        assert!(
            game.score_file(OURS).is_none(),
            "orb's own file was written by a run nothing could hit",
        );

        // And the game carried on past it, scene 7 to 1: the run reached its title menu, which is
        // `WriteDataToFile` checking its open rather than the game stopping on it.
        game.frames_until(
            "the title menu after the result screen",
            A_WHOLE_RUN,
            || game.image().scene() == Scene::FrontEnd,
        );
    });
}
