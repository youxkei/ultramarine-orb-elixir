//! 完全無欠モード over a laid-out 妖々夢: the question that starts it, a chapter, a death and a retry.
//!
//! **This is what `Th07::rewinds` rests on.** Everything else about that game is a unit test over a laid-out
//! address space — one method, one answer — and none of those can say that the *mode* holds together there:
//! the chapter a snapshot is taken as covers 14MB of this game's own data rather than 紅魔郷's 2MB, the
//! ranges a restore must leave alone are this game's sound system, the retry menu is drawn in a face the
//! machine lent because 妖々夢 ships no font, and the card record has to come back through the seal this
//! game checks itself with. A `true` there without this file would be orb claiming the mode on the strength
//! of unit tests.
//!
//! **The game plays the game's part and this presses keys.** What moves the run along is the laid-out game —
//! see `fake::th07` — and what this file does is press, run frames, and read back the game's own memory and
//! what orb put in the log. Nothing here calls orb to move anything.
//!
//! The order is the order somebody playing meets it, for the reason `pointdevice_run.rs` gives: the parts
//! have to agree with each other, and nothing that tests them apart can say whether they do.

use crate::fake::th07::Fake;
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as `th07.rs`'s own launch declares it.
const WORK_US: i64 = 700;

/// The run this e2e test plays: Normal, Reimu A, from stage one — the difficulty and the shot as the
/// game's own front end leaves them.
const DIFFICULTY: i32 = 1;
const STAGE: i32 = 0;

/// Two of the retry menu's own ways on, in the language a launch on this machine comes out in: the chapter
/// again, and the run given up. Read by their own text, the way `pointdevice_run.rs` reads them.
const AGAIN: &str = "チャプターをやり直す";
const GIVEN_UP: &str = "タイトルに戻る";

/// And what the chapter it is offering is called: the waves of a stage are one run of chapters and its own
/// start is the first of them, so a stage with no midstage boundary measured has this one and no other.
const CHAPTER: &str = "MIDSTAGE 1";

/// The numbers a stage of it starts with, which is what a chapter has to put back.
const LIVES: f32 = 2.0;
const BOMBS: f32 = 3.0;
const POWER: f32 = 96.0;
const SCORE: u32 = 1_234_560;

#[test]
fn a_pointdevice_run_of_th07_is_chosen_chaptered_lost_and_retried() {
    in_its_own_process(|| {
        // `verbose`, because half of what this reads back is written at that level.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-pointdevice",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        // ── 1. The mode. A game orb can rewind starts in 完全無欠モード and is asked which at its own
        // title menu, which is what `Th07::rewinds` answering `true` turns on.
        assert!(
            log.said(&format!(
                "mode: {} to start with; the menu asks which",
                Mode::Pointdevice
            )),
            "the launch did not start in the mode orb is for: {:?}",
            log.lines(),
        );

        // The overlay first, which is orb's own rule about asking anything — and here it is a face the
        // machine lent, 妖々夢 shipping no `font.ttf`. See `docs/adr/0019`.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));

        // The title menu up and past the frames its own press test refuses, with the cursor on the item
        // that starts a run.
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        // Pressed again and again rather than once, which is what somebody at the keyboard does: orb decides
        // whether to hold a press back at the end of an update and acts on it in the next, so the first frame
        // the cursor is somewhere orb asks about is a frame whose press goes to the game.
        for _ in 0..4 {
            game.press(keys::Z);
        }
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press did not put the question up: {:#?}",
            log.lines(),
        );

        // What the question drew, read off the screen: the title of it, in the face the machine lent.
        game.one_frame();
        assert_eq!(
            game.says(title(Menu::Run, LANGUAGE)).len(),
            1,
            "the mode question is not on the screen",
        );

        // Answered on the keyboard, which is the hand this run has.
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        assert!(
            log.said(&format!("mode: {}", Mode::Pointdevice)),
            "the mode was not settled: {:?}",
            log.lines(),
        );

        // ── 2. The run. The game builds it, which is what a laid-out one does when this says so: the
        // scene, the stage, the numbers behind the pointer the build allocates, and the panel it puts over
        // them — whose own tile is what the strips either side of the mark below are painted with.
        game.image().is_playing(STAGE, DIFFICULTY);
        game.image().has_a_runs_numbers();
        game.image().has_lives_and_bombs(LIVES, BOMBS);
        game.image().shot_is_at_power(POWER);
        game.image().the_run_has_scored(SCORE, 0);
        game.image().has_a_panel();
        game.image().paints_its_panel();
        // And it goes on by itself from here, which is what a stage does: both of its clocks a frame further
        // on with every frame.
        game.the_run_goes_on();
        game.frames_until("the run", 8, || game.state().playing);

        // ── 3. The stage's first chapter, which is its own start: nothing of the midstage table is
        // measured yet, so every chapter of this game is a stage's start until a tuning run says otherwise.
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // And the count of lives painted over, there being something to go back to: the mark says a death
        // costs the chapter rather than a life — and the strips either side of it are painted with the
        // panel's own tile, which orb took out of the vm the game lays its background with.
        game.one_frame();
        assert!(
            !game.says("DISABLE").is_empty(),
            "the lives were not marked as disabled: {:?}",
            log.lines(),
        );
        assert!(
            log.said("lives: the panel's own tile is what the strips are painted with"),
            "the mark was painted flat over the panel: {:?}",
            log.lines(),
        );

        // ── 4. The death. The game counts one, takes the life, and puts the player in its own dying state —
        // and it is the *count* orb reads a death off, which is what makes a death bomb no death at all: that
        // number moves only once the window for one has closed.
        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_is_dying();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));

        // The retry menu, drawn over the frozen game and read off the screen: its own question, and the
        // chapter the run was in under it. In the face the machine lent, which is what makes this the one
        // e2e test that says orb's own screens are readable in 妖々夢 at all.
        game.one_frame();
        assert_eq!(
            (
                game.says(AGAIN).len(),
                game.says(GIVEN_UP).len(),
                game.says(CHAPTER).len(),
                game.says("RETRY 0").len(),
            ),
            (1, 1, 1, 1),
            "the menu over the frozen game is not the one the mode puts there: {:?}",
            log.lines(),
        );

        // ── 5. The retry. Answered on the keyboard, and what comes back is the chapter's own numbers:
        // this game seals them, so a restore that put them back without the seal would have the game
        // wreck itself a moment later — and the numbers being right is what says the seal was called.
        game.press_until(keys::Z, "the chapter put back", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let state = game.state();
        assert_eq!(
            (state.lives, state.bombs),
            (LIVES as i8, BOMBS as i8),
            "the chapter did not put the run's numbers back",
        );
        assert!(state.playing, "the run was not left playing");
    });
}
