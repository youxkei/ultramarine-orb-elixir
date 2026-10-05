//! **The screen a run of th06c ends at is the game's to walk, and orb writes one state into it.**
//!
//! That screen is several one after another, off the jump table at `0x1400535c8`: the name entry a played
//! run opens in (state 9, `0x14004fde9`), the stats the name's decide opens (15, which takes a decide of its
//! own once up ninety frames), the thirty frames those fade for (16), the question about saving a replay
//! (10) and that question with its cursor live (14), and the way out (17).
//!
//! **Only the question is orb's business.** A Pointdevice run has nothing the *game* could put in a replay —
//! its record is orb's own file — so `Th06c::skip_replay_prompt` writes the way out over that one state and
//! over no other, which is the frame the game would have asked on. The name entry and the stats before it
//! belong to whoever just finished the run.
//!
//! **Which is why the write is at the question and not where the screen comes up.** A write on the frame the
//! screen is built would cover the name entry, and a name entry sent away is a run's own score with nobody's
//! name on it: th06 1.02h lost both that screen and the stats screen inside two frames to a ranking asked
//! for on the wrong one, which is `the_screen_a_finished_run_ends_at.rs`'s own subject.

use orb_config::LogLevel;
use orb_core::game::th06c::image::{
    RESULT_GRID_ACROSS, RESULT_LEAVING, RESULT_NAME_BYTES, RESULT_PLACES_RANKED,
    RESULT_SAVE_REPLAY_QUESTION, RESULT_STATS, RESULT_STATS_FADING, RESULT_THE_END_ITEM,
    RESULT_THE_ENTRY_TAKES_A_KEY_AFTER, RESULT_TYPING_A_NAME,
};
use orb_core::game::th06c::score_file;
use orb_sim::keys;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames, and how many frames a whole run of six
/// needs: `--clear` is how a result screen is reached at all in this mode, so the shape is
/// `a_clear_on_demand_of_th06c`'s.
const STAGE_FRAMES: u32 = 330;
const STAGES: u32 = 6;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES * 2;

/// How many frames the screens of a finished run are given to be walked: the stats alone ignore a press for
/// ninety of them and the fade is thirty more.
const THE_SCREENS: u32 = 600;

/// The line orb writes where it has written the question away.
const NO_REPLAY: &str = "result: no replay of the game's own is offered for a run with chapters";

/// A run in Pointdevice mode played to its ending under `--clear`, and the screen it ends at after it.
fn a_cleared_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.block_replay_save = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    // On Normal, an Easy run ending after its fifth stage — see `the_way_a_stage_of_th06c_ends`.
    game.image().the_difficulty_is(NORMAL);
    game.in_a_run_in_the_stage(0);
    game.frames_until("the screen the run ends at", A_WHOLE_RUN, || {
        game.image().the_state_the_screen_a_run_ends_at_is_in() != 0
    });
    game
}

/// **Every state that screen came to, in the order it came to them**, with the decide tapped every other
/// frame until the screen is on its way out.
fn walked(game: &Fake) -> Vec<i32> {
    let mut seen = vec![game.image().the_state_the_screen_a_run_ends_at_is_in()];
    let mut down = false;
    for _ in 0..THE_SCREENS {
        if *seen.last().expect("a state") == RESULT_LEAVING {
            break;
        }
        down = !down;
        game.keyboard().set(keys::Z, down);
        game.frame();
        let state = game.image().the_state_the_screen_a_run_ends_at_is_in();
        if state != *seen.last().expect("a state") {
            seen.push(state);
        }
    }
    game.keyboard().set(keys::Z, false);
    seen
}

/// A finished run's screen opens in the name entry, which is the state the game builds it in.
#[test]
fn the_screen_a_finished_run_of_th06c_ends_at_opens_in_its_name_entry() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screen-a-run-ends-at");
        assert_eq!(
            game.image().the_state_the_screen_a_run_ends_at_is_in(),
            RESULT_TYPING_A_NAME,
            "the screen the run ended at did not open in the name entry",
        );
        // And nothing of orb's has been written into it yet, which is what the write's own frame is about.
        assert!(
            !game.log().said(NO_REPLAY),
            "the question was written away on the frame the screen came up, which covers the name entry",
        );
    });
}

/// And the screens after it are the game's own, walked by presses, with orb's one write at the question.
#[test]
fn the_screens_of_a_finished_run_of_th06c_are_walked_to_the_question_orb_writes_away() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screens-a-run-ends-at");
        let seen = walked(&game);

        assert_eq!(
            seen,
            vec![
                RESULT_TYPING_A_NAME,
                RESULT_STATS,
                RESULT_STATS_FADING,
                RESULT_LEAVING
            ],
            "the screens of a finished run did not walk the name entry, the stats and their fade into the \
             way out",
        );
        // **And the question never stood for a frame of its own**, which is where orb's one write lands: it
        // is made on the frame the game put that state there, so no frame of the screen is ever the
        // question's — the read is `Th06c::skip_replay_prompt`'s own gate.
        assert!(
            !seen.contains(&RESULT_SAVE_REPLAY_QUESTION),
            "the question about saving a replay stood for a frame of its own: {seen:?}",
        );
        assert!(
            game.log().said(NO_REPLAY),
            "the question about saving a replay was not written away:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// **The name entry the way case 9 of 0x14004ff40 walks it**: up with eight spaces and the cursor on the
/// grid's first cell, deaf to a key for its first thirty updates, a decide typing the cell the cursor is on,
/// the eighth character putting the cursor on the end item, and a decide there leaving for the stats.
#[test]
fn the_name_entry_of_th06c_types_a_character_a_decide_and_leaves_on_its_end_item() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-name-entry");
        assert_eq!(
            game.image().the_name_the_entry_holds(),
            "",
            "the entry came up holding a name"
        );
        game.frames(THE_ENTRY_HEARS_NOTHING_FOR);
        game.press(keys::Z);
        assert_eq!(
            (
                game.image().the_state_the_screen_a_run_ends_at_is_in(),
                game.image().the_name_the_entry_holds()
            ),
            (RESULT_TYPING_A_NAME, "A".to_owned()),
            "a decide in the entry did not type the cell the cursor is on",
        );
        for _ in 1..RESULT_NAME_BYTES {
            game.press(keys::Z);
        }
        assert_eq!(
            game.image().the_entrys_cursor(),
            RESULT_THE_END_ITEM,
            "the eighth character did not put the cursor on the end item",
        );
        game.press(keys::Z);
        assert_eq!(
            game.image().the_state_the_screen_a_run_ends_at_is_in(),
            RESULT_STATS,
            "a decide on the end item did not leave the entry for the stats",
        );
    });
}

/// And the cursor walks the grid round, over the cells that are spaces.
#[test]
fn the_cursor_of_th06cs_name_entry_walks_its_grid_round() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-name-entry-cursor");
        game.frames(THE_ENTRY_HEARS_NOTHING_FOR);
        game.press(keys::LEFT);
        assert_eq!(
            game.image().the_entrys_cursor(),
            RESULT_GRID_ACROSS - 1,
            "left from the first cell did not go round to the end of its row",
        );
        game.press(keys::UP);
        assert_eq!(
            game.image().the_entrys_cursor(),
            RESULT_THE_END_ITEM,
            "up from the first row did not go round to the last",
        );
    });
}

/// And a run whose score ten of the file's own records are above is asked for no name at all.
#[test]
fn a_run_of_th06c_placed_below_the_ranking_is_asked_for_no_name() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-below-the-ranking",
            Work::wandering(WORK_US),
            |config| {
                config.fast_clear = true;
                config.block_replay_save = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let mut records = vec![score_file::th6k()];
        records.extend(
            (0..RESULT_PLACES_RANKED)
                .map(|_| score_file::record(b"HSCR", &A_SCORE_ABOVE_ANY_RUN.to_le_bytes())),
        );
        game.its_records_are(&records);
        game.stages_last(STAGE_FRAMES);
        game.fires_at_the_player(false);
        game.image().the_difficulty_is(NORMAL);
        game.in_a_run_in_the_stage(0);
        game.frames_until("the stats", A_WHOLE_RUN, || {
            game.image().the_state_the_screen_a_run_ends_at_is_in() == RESULT_STATS
        });
        assert_eq!(game.image().the_name_the_entry_holds(), "");
    });
}

/// The updates the entry hears no key for, and a score no run here reaches.
const THE_ENTRY_HEARS_NOTHING_FOR: u32 = RESULT_THE_ENTRY_TAKES_A_KEY_AFTER as u32 + 1;
const A_SCORE_ABOVE_ANY_RUN: u32 = 999_999_990;
const NORMAL: i32 = 1;

/// And that state is said in the log whenever it changes, which is how a walk of those screens is read back
/// off a launch of the real game.
#[test]
fn the_state_the_screen_a_finished_run_of_th06c_ends_at_is_in_is_said_as_it_changes() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screen-said");
        walked(&game);
        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| line.contains("result: the screen a run ends at"))
            .collect();
        assert!(
            said.len() >= 2,
            "the screen's own states were not said as they changed: {said:?}",
        );
    });
}
