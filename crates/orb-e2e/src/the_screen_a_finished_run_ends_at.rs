//! **The screen a run's own end arrives on is the game's to walk, and orb writes one state into it.**
//!
//! What each e2e test holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! `ResultScreen` is what a finished run reaches, and it is several screens one after another: the
//! high-score name entry `ResultScreen::RegisterChain` starts it in, the stats screen, and then the
//! question about saving a replay. Only the last of those is orb's business — a pointdevice run has
//! nothing to put in a replay, so the state is written past that question and no part of it is drawn, which
//! is `a_clear_on_demand.rs`'s subject. Every screen before it belongs to whoever just finished the run.
//!
//! **Measured, and this is the run that asked for these e2e tests.** Two sessions in `th06/orb.log`, an
//! Extra clear and a stage 6 clear, both this shape:
//!
//! ```text
//! run ended after 27 retries
//! f151268 scene=7->2                                                  the result screen
//! score: pointdevice_score.dat opened in place of the game's own, read  its added callback
//! score: the ranking was not built after 240 update(s); nothing written
//! score: pointdevice_score.dat opened in place of the game's own, write its deleted callback
//! f151270 scene=1->7                                                  the title menu
//! ```
//!
//! **Two frames**, with the name entry and the stats screen inside them: nobody saw either. What did it was
//! the ranking a run's end asks for — the front end it is asked of is not up on the result screen, so the
//! whole allowance of updates went there and the request was then undone, and undoing it wrote
//! `RESULT_SCREEN_STATE_EXITING` into the screen that *was* up. One screen in 紅魔郷 reached two ways, and
//! the same field in the same object either way.
//!
//! A run that finished needs no ranking built for it: the screen it finished at is what writes the score
//! file, by the deleted callback the line above is. `the_score_file.rs` is where that half is.

use crate::fake::th06::{CARD, Fake, STAGES, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, attempts_in, result_state};

/// How long each stage of these runs is, in frames, and how many frames a whole run of six needs — the
/// same shape `a_clear_on_demand.rs` uses, since `--clear` is how a result screen is reached at all in
/// 完全無欠モード.
const STAGE_FRAMES: u32 = 300;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// The two names an open of the score file can land in.
const THEIRS: &str = "score.dat";

/// A run in 完全無欠モード, finished: `--clear` to the ending, and the result screen after it.
fn a_cleared_pointdevice_run(name: &str) -> Box<Fake> {
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

/// The screen the run finished at, on the frame it was built.
///
/// One frame past the one the scene was asked for on, which is where `ResultScreen::RegisterChain` runs: the
/// supervisor acts on a scene that has been asked for on the update after.
///
/// # Panics
/// Naming the state it came up in, where that is not the one a run somebody played arrives on.
fn arrives_on_the_name_entry(game: &Fake) {
    game.frames_until("the result screen", A_WHOLE_RUN, || {
        game.image().scene() == Scene::Result
    });
    game.frame();
    assert_eq!(
        game.image().result_screen_state(),
        result_state::WRITING_HIGHSCORE_NAME,
        "the screen a finished run arrived on is not the name entry the game registered it in",
    );
}

/// And the screen still standing on it, frame after frame, until the game's own walk moves it off — with
/// the state it moved to, which is what each e2e test below says about its own mode.
///
/// The stretch itself rather than a count of frames, because a count is this game's own — the real name
/// entry stands until somebody has typed a name — and what is being asserted is that the screen is still
/// there and still on it the whole way.
///
/// # Panics
/// Where the screen went down while the name entry was up, which is a screen taken away from whoever was
/// standing at it.
fn stands_until_it_moves_on(game: &Fake) -> i32 {
    game.frames_until("the name entry walked past", A_WHOLE_RUN, || {
        if game.image().result_screen_state() != result_state::WRITING_HIGHSCORE_NAME {
            return true;
        }
        assert_eq!(
            game.image().scene(),
            Scene::Result,
            "the screen the run finished at went down while its name entry was up",
        );
        false
    });
    game.image().result_screen_state()
}

/// A finished 完全無欠モード run's name entry stands, and the one state orb writes into that screen is
/// written at the question after it.
///
/// Which is the whole of what orb has to do with `ResultScreen`: the question is one this run cannot answer
/// and the screens before it are ones it has no business touching. Both halves here, because the first says
/// nothing alone — a launch that never wrote anything into the screen would pass it and leave the question
/// standing.
#[test]
fn a_finished_runs_name_entry_stands_and_only_the_question_is_written_past() {
    in_its_own_process(|| {
        let game = a_cleared_pointdevice_run("the-screen-a-finished-run-pointdevice");
        arrives_on_the_name_entry(&game);
        // The way out, and not the question the game walked to: orb writes past that inside the same frame
        // it arrives, which is what keeps it from being drawn — `a_clear_on_demand.rs` is where what the
        // write buys is asserted. So the state a frame of the loop can see the screen in never is the
        // question, and orb's own line is what says which state it wrote over.
        assert_eq!(
            stands_until_it_moves_on(&game),
            result_state::EXIT,
            "the state orb wrote is not the one the game's own way out of that screen is",
        );
        assert!(
            game.log()
                .said("result: no replay is offered for a run with chapters"),
            "the write was made somewhere other than at the question about saving a replay:\n  {}",
            game.log().lines().join("\n  ")
        );
        // And the screen goes down from there, which is the game's own case for that state: the title menu
        // after it, and the run over.
        game.frames_until("the title menu after it", A_WHOLE_RUN, || {
            game.image().scene() == Scene::FrontEnd
        });
    });
}

/// And a finished レガシーモード run's, whose question nothing writes past: the screen walks all of itself
/// out, and what it wrote is what the run counted.
///
/// The mode makes two differences and this is where both are visible. Nothing of orb's is in the way of that
/// screen at all — no chapters, so no question about a replay — and the file it writes is the game's own. So
/// what this holds against the run is the count in that file: a run that finished writes by the deleted
/// callback of the screen it finished at, and needs no ranking built for it to do that.
#[test]
fn a_finished_legacy_run_walks_its_own_screen_out_and_that_screen_writes() {
    in_its_own_process(|| {
        let game = Fake::attach("the-screen-a-finished-run-legacy", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.in_a_legacy_run();
        // Past the card its boss puts up, which is where the game counts an attempt at one: the number this
        // run has to have written down is that count and no other, orb adding none in this mode.
        game.frames_until("the card its boss puts up", 900, || {
            game.image().card_attempts(CARD) == 1
        });
        // Out of lives, which is how the game ends a run it is not asked to end: a death costs a life here
        // and nothing is rewound.
        game.forget_score_file_opens();
        while game.state().lives >= 0 {
            game.hit();
            game.frame();
        }
        game.frames_until("the run over", 60, || !game.state().in_run);

        arrives_on_the_name_entry(&game);
        // The question itself here, nothing of orb's having written over it: a run in this mode has a replay
        // worth saving, so the screen that offers one is a screen it reaches.
        assert_eq!(
            stands_until_it_moves_on(&game),
            result_state::SAVE_REPLAY_QUESTION,
            "the name entry did not hand over to the question about saving a replay",
        );
        // And the screen draws it and walks itself out.
        game.frames_until("the title menu after it", A_WHOLE_RUN, || {
            game.image().scene() == Scene::FrontEnd
        });

        // What the screen wrote on the way down, which is one write and the game's own file: `WriteScore`
        // from the deleted callback, with what is in memory being this run's own record — the parses that
        // would have put the file's back over it are the ones `AddedCallback` skips at a run's end.
        let opens = game.score_file_opens();
        let writes: Vec<&str> = opens
            .iter()
            .filter(|open| open.write)
            .map(|open| open.path.as_str())
            .collect();
        assert_eq!(
            writes,
            vec![THEIRS],
            "the screen the run finished at did not write the game's own file exactly once",
        );
        let written = game.score_file(THEIRS).expect("the file that screen wrote");
        assert_eq!(
            attempts_in(&written, CARD),
            1,
            "the file holds a count this run did not make",
        );

        // And no ranking was built for it: the screen it finished at is what writes, so there is nothing
        // left for one to carry.
        assert!(
            !game.log().said("score: the ranking built and taken down"),
            "a ranking was built for a run that finished at the result screen as well:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            !game.log().said("score: the ranking was not built"),
            "a ranking was asked for where no front end was up:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
