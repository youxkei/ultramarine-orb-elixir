//! **What a stage transition carries, and what only the start of a run puts in place.**
//!
//! `GameManager::AddedCallback` is one function reached two ways, and the whole of the difference is its
//! own first condition: `g_Supervisor.curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT`. A run's first stage
//! takes the branch — the score file's read, the lives, the bombs, the power, the deaths, the arcade region
//! and the box the player is held inside — and a stage transition takes the two-line `else`, `guiScore =
//! score` and `nextScoreIncrement = 0`, and carries everything the run had. Which is what a run *is*: six
//! stages with one set of numbers walked through them.
//!
//! **Read out of the game's own code rather than off this tree**, because the fake game is both the writer
//! and the reader of the memory orb is driven through: a transition that put a stage's numbers back would
//! be a transition every e2e test here agreed with. The route from an address to the function it is the
//! start of is *Re-deriving it* in
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md), and the
//! condition above is at `src/GameManager.cpp`'s `AddedCallback`.
//!
//! **Which condition the fake is on needs nothing kept beside its memory.** `build` calls
//! `orb_core::runtime::stage_begun` before it writes the supervisor's copy, so at the moment a stage's numbers go in the
//! scene is still `Scene::Rebuilding` for a transition and already `Scene::Playing` for a run's first
//! stage — the game's own `curState`, read the way the game reads it.
//!
//! レガシーモード throughout, so that nothing of orb's is between the run and the game: a chapter put back
//! over a death would be a second answer to what the next stage begins with.

use crate::fake::th06::{
    FRESH, Fake, INVULNERABLE_AFTER_SPAWNING, Open, PLAYER_AREA_SIZE, PLAYER_AREA_TOP_LEFT,
    RANK_AT_A_RUNS_START, SHAKE_FRAMES, the_run,
};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{Scene, Screen, item};
use orb_sim::keys;

/// How long each of this game's stages runs here, in its own frames: long enough for a life, a bomb and a
/// power item to be spent inside the first one — which means past the frames a stage's own start cannot be
/// killed in, [`INVULNERABLE_AFTER_SPAWNING`] — and short enough that the transition is not a wait.
const STAGE_FRAMES: u32 = INVULNERABLE_AFTER_SPAWNING as u32 + 100;

/// A launch with orb saying everything it does, which is how the trip out of a run is read back.
fn launched(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    })
}

/// A run started from the title menu, for the second one a launch plays: the mode has been answered
/// already, so the press at the title goes straight to the game's own shot type select.
///
/// The cursor is walked back to the item that starts a run first, because it is not where a launch left
/// it: orb takes a run that ended anywhere but the result screen through the game's own ranking, and
/// getting there is the cursor moved to the item the ranking is behind.
fn starts_another_run(game: &Fake) {
    game.at_the_title_menu();
    let cursor = game.image().front_end_now().cursor;
    for _ in item::GAME_START..cursor {
        game.press(keys::UP);
    }
    assert_eq!(
        game.image().front_end_now().cursor,
        item::GAME_START,
        "the cursor is not on the item that starts a run",
    );
    game.press_until(keys::Z, "the shot type select", || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.frames_until("the stage built", 8, || game.state().playing);
}

/// A life lost to a bullet, past the frames a stage's own start cannot be killed in.
///
/// The wait is the point rather than politeness: `Player::AddedCallback` leaves the player invulnerable for
/// [`INVULNERABLE_AFTER_SPAWNING`] updates — see `the_player_a_stage_starts.rs` — so a hit inside
/// them is a hit the game does nothing with.
fn loses_a_life(game: &Fake) {
    let killable = INVULNERABLE_AFTER_SPAWNING as u32;
    game.frames_until("the stage killable again", killable + 60, || {
        game.state().stage_frames >= killable
    });
    game.hit();
    game.frame();
}

/// Every open of the score file that was for reading.
fn reads(opens: &[Open]) -> Vec<String> {
    opens
        .iter()
        .filter(|open| !open.write)
        .map(|open| open.path.clone())
        .collect()
}

/// A run carries its lives, its bombs and its power across a stage boundary.
///
/// Read out of `GameManager::AddedCallback`: `livesRemaining` and `bombsRemaining` are written **nowhere in
/// it**, on either branch — what puts them there is the front end, out of `g_Supervisor.defaultConfig` —
/// and `currentPower = 0` is inside the branch a transition does not take. So the three numbers a stage is
/// played with are the run's and not the stage's, and a transition that put them back would hand a run six
/// fresh lives.
#[test]
fn a_run_carries_its_lives_bombs_and_power_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = launched("a-stage-transition-what-a-run-carries");
        game.stages_last(STAGE_FRAMES);
        game.in_a_legacy_run();

        // A life to a bullet, a bomb spent on the way out of trouble, and a power item collected: all
        // three of the numbers, because a transition that put one of them back would be as wrong as one
        // that put all three back and no assertion about the other two would say so.
        loses_a_life(&game);
        game.bombs();
        game.collects_a_power_item();
        let before = game.state();
        assert_eq!(
            (before.lives, before.bombs, before.power),
            (FRESH.0 - 1, FRESH.1 - 1, FRESH.2 + 1),
            "the first stage did not spend the life, the bomb and the power this is about",
        );
        assert_eq!(
            before.stage, 0,
            "the run is not in the stage it spent them in",
        );

        // The transition, which is one frame with the next stage built inside it.
        game.frames_until("the stage after the first", STAGE_FRAMES + 60, || {
            game.state().stage == 1
        });
        let after = game.state();
        assert_eq!(
            (after.lives, after.bombs, after.power),
            (before.lives, before.bombs, before.power),
            "the stage after the first was handed a run's numbers put back rather than the run's own",
        );
        // And the deaths with them, which the result screen is the only reader of: those a run has always
        // carried, and they are here so that a fix which moved them into the branch would be caught.
        assert_eq!(
            after.deaths, before.deaths,
            "the stage after the first forgot the death the run had",
        );
    });
}

/// And it reads the score file once, at the run's start, rather than once per stage.
///
/// The read is `ResultScreen::OpenScore("score.dat")` inside the same branch, so a transition makes none.
/// **What it costs is not the open**: that read is also what calls `Th06::set_captures`, so the record of
/// spell cards was written back from the file as often as the file was opened — and a resume plays a run's
/// buttons in with that record held across the playback (`resume::hold_captures`). A run whose stage moves
/// each re-read the file is a run whose captures each landed on the file's version.
#[test]
fn a_stage_transition_makes_no_read_of_the_score_file() {
    in_its_own_process(|| {
        let game = launched("a-stage-transition-the-score-file");
        game.stages_last(STAGE_FRAMES);
        // The front end's own read at `MainMenu::AddedCallback` is behind us and is not what this is
        // about — see `the_score_file.rs`, which is where which file each read lands in is.
        game.at_the_title_menu();
        game.forget_score_file_opens();

        game.in_a_legacy_run();
        let at_the_start = reads(&game.score_file_opens());
        assert_eq!(
            at_the_start.len(),
            1,
            "the run's first stage did not read the score file exactly once: {at_the_start:?}",
        );

        game.frames_until("the stage after the first", STAGE_FRAMES + 60, || {
            game.state().stage == 1
        });
        assert_eq!(
            reads(&game.score_file_opens()),
            at_the_start,
            "the stage transition read the score file, which is a read the game makes once a run",
        );
    });
}

/// A run is started at the difficulty's own rank, and a stage transition puts the sub-rank back and the rank
/// not.
///
/// `mgr->rank = g_DifficultyInfo[difficulty].rank` is in the branch a transition does not take — after an
/// earlier `rank = 8` in the same branch, which nothing between the two reads — while `mgr->subRank = 0` sits
/// outside the condition and so happens at every stage. Both are numbers orb reads:
/// `Th06::reproduction` carries them, `Th06::run_state` writes the rank back on a resume, and until they were
/// laid out every assertion about a restored rank was comparing zero with zero.
#[test]
fn a_run_starts_at_the_difficultys_own_rank_and_a_transition_leaves_it_alone() {
    in_its_own_process(|| {
        let game = launched("a-stage-transition-the-rank");
        game.stages_last(STAGE_FRAMES);
        game.in_a_legacy_run();
        let started = unsafe { Th06.reproduction() };
        assert_eq!(
            (started.rank, started.sub_rank),
            (RANK_AT_A_RUNS_START, 0),
            "the run did not start at the rank its difficulty is played at",
        );

        game.frames_until("the stage after the first", STAGE_FRAMES + 60, || {
            game.state().stage == 1
        });
        let moved = unsafe { Th06.reproduction() };
        assert_eq!(
            (moved.rank, moved.sub_rank),
            (RANK_AT_A_RUNS_START, 0),
            "the stage after the first was not played at the rank the run had reached",
        );
    });
}

/// The box the player is held inside is the game's own and not the arcade region's.
///
/// `playerMovementAreaTopLeftPos` and `playerMovementAreaSize` — `(8, 16)` and `(368, 416)` — sit beside the
/// arcade region's `(32, 16)` and `(384, 448)` in the same branch and are a different rectangle:
/// `Player::HandlePlayerInputs` clamps `positionCenter` to this one, and `Player::AddedCallback` measures a
/// stage's first position from that one. Both halves are here because orb reads both — `Th06::reproduction`
/// carries the box, and where the player really stops is what the box is *for*.
#[test]
fn a_run_holds_its_player_inside_the_games_own_movement_area() {
    in_its_own_process(|| {
        let game = launched("a-stage-transition-the-movement-area");
        game.in_a_legacy_run();
        assert_eq!(
            unsafe { Th06.reproduction() }.player_area,
            (PLAYER_AREA_TOP_LEFT.1, PLAYER_AREA_SIZE.1),
            "the run was given the arcade region as the box its player is held inside",
        );

        // And the floor of it, which is where the player stops: far enough down that the clamp is what
        // stopped them and not the frames running out.
        let floor = PLAYER_AREA_TOP_LEFT.1 + PLAYER_AREA_SIZE.1;
        game.keyboard().set(keys::DOWN, true);
        game.frames(INVULNERABLE_AFTER_SPAWNING as u32);
        game.keyboard().set(keys::DOWN, false);
        assert_eq!(
            unsafe { Th06.reproduction() }.player.1,
            floor,
            "the player was let past the foot of the box the game holds them inside",
        );
    });
}

/// A new run puts the arcade region back where the game has it, and the run's numbers back to a run's own.
///
/// The region is written in that same branch — `(32, 16)` and `(384, 448)`, `src/GameManager.cpp` — and by
/// nothing per stage, which is the whole reason a bomb's screen shake can reach the stage after the one that
/// started it: the shake writes the region every frame and only puts it back on the frame it removes itself
/// on. A run given up while one is still running leaves it moved, and what is asked here is that the *next*
/// run does not begin inside it.
///
/// **What the new run's own region write is read through is the player's first position**, and not the region
/// itself. `Player::AddedCallback` measures `arcadeRegionSize.x / 2` across and `arcadeRegionSize.y - 64`
/// down, so the position is the region as it stood at the moment the stage's numbers went in — which is the
/// only moment the question is about. The region a frame later is a different number, because
/// `GameManager::DeletedCallback` cuts Stage, BulletManager, Player, EnemyManager, EffectManager and Gui and
/// **not** `ScreenEffect`: a shake the last run left running is still a job of the chain, and it writes the
/// region again on the new stage's first update.
#[test]
fn a_new_run_puts_the_arcade_region_and_the_runs_own_numbers_back() {
    in_its_own_process(|| {
        let game = launched("a-stage-transition-a-new-run");
        let field = Th06.play_area();
        game.in_a_legacy_run();
        // Where a stage starts the player when the region is the game's own, which is what the second run
        // has to arrive back at.
        let starts_at = unsafe { Th06.reproduction() }.player;

        // A life lost and a bomb, and the run given up inside the shake's own frames: one that had run out
        // would have put the region back itself.
        loses_a_life(&game);
        game.bombs();
        game.frames(SHAKE_FRAMES as u32 / 2);
        assert!(
            game.image().shaking_the_screen(),
            "the shake was over before the run was, so there is nothing left moved to put back",
        );
        assert_ne!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "the shake left the arcade region where the stage had it",
        );
        let given_up = game.state();
        assert_eq!(
            (given_up.lives, given_up.deaths),
            (FRESH.0 - 1, 1),
            "the run being given up had not lost the life this is about",
        );

        // `esc` and then やめる, and orb's trip through the game's own ranking after it: what the run
        // counted is written on the way out, and the front end comes back up behind that.
        game.gives_the_run_up_at_its_own_pause();
        game.frames_until("the title menu after the run", 600, || {
            game.image().scene() == Scene::FrontEnd
                && game.image().front_end_now().screen == Screen::Title
        });
        assert_ne!(
            game.image().arcade_region_size(),
            (field.width, field.height),
            "something between the run ending and the title menu put the arcade region back",
        );

        // And the run after it, which is the branch this is about.
        starts_another_run(&game);
        assert_eq!(
            unsafe { Th06.reproduction() }.player,
            starts_at,
            "the new run started its player somewhere a stage does not, which is the arcade region the \
             last run's bomb left moved being what it was measured from",
        );
        let started = game.state();
        assert_eq!(
            (started.lives, started.bombs, started.power, started.deaths),
            (FRESH.0, FRESH.1, FRESH.2, 0),
            "the new run began with the numbers the last one was left on",
        );
    });
}
