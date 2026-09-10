//! **The front end of th08 walked from its title into a run, by the screen numbers a recipe of this game
//! presses at.**
//!
//! `Game::front_end_screen` answers the front end's `+0x14428` and `front_end_cursor` its `+0x1442c`, and
//! the three numbers those come back with are what both of this game's recipes are written against: 0 is
//! the title menu, whose item 0 starts a run; 4 is the difficulty select, which comes up on Normal; and 5
//! is the team select, 「人と妖怪の選択の刻」. `orb-core`'s `game/th08/asm.md` has where each was read.
//!
//! **Which is what an e2e test over this game can say about the front end and the whole of it**: what a
//! press does on each of those screens is the laid-out game's, so nothing here holds the real game to a
//! screen it does not have. What it does hold is orb — that a recipe's `until: { screen: 4 }` is answered
//! off the field that screen is really in, that a cursor pressed to is the cursor orb reads, and that a
//! decide at the last of them is a run orb sees start.
//!
//! **A press as the team select comes up is no press at all**, which a launch of the real game measured — a
//! recipe that pressed once left the game on that screen with the scene still the front end's — and the
//! screen's own code says why: it comes up over eight updates and reads its decide from the one after them
//! (0x46a92d through 0x46a939). So a recipe presses it twice, and the second press is the one it reads.

use crate::fake::th08::{DIFFICULTY_NORMAL, Fake, TITLE_ITEM_PRACTICE, TITLE_ITEM_START};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th08::Th08;
use orb_core::game::th08::image::{Scene, Screen};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen is given to answer a press, which is well past the ten the game's own gate
/// holds one off for.
const FRAMES_FOR_A_SCREEN: u32 = 120;

/// And how many frames the team select is given to come up before its press, which is past the eight
/// updates 0x46a92d counts.
const THE_TEAM_SELECT_COMES_UP_OVER: u32 = 10;

/// The key this game reads as *decide* and the two it reads as a direction, as its own fold has them: `Z`
/// gives the shot bit at 0x43dbd7, and the arrows give the up and down bits at 0x43dacf and 0x43daf1.
const DECIDE: u8 = keys::Z;
const DOWN: u8 = keys::DOWN;

fn at_its_title(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    // Left to itself rather than placed, which is the launch a recipe's first step waits out: the title
    // comes up some frames in, and `wait_until: { screen: 0 }` is what waits for it — **and then builds itself
    // over the updates it reads no decide in**, which is what a recipe's first press is repeated past. See
    // `the_title_menu_of_th08_walked`.
    game.frames_until("the title menu came up", FRAMES_FOR_A_SCREEN, || {
        (unsafe { Th08.front_end_screen() }) == Some(Screen::Title as i32)
    });
    game.frames_until(
        "the title menu taking a press",
        FRAMES_FOR_A_SCREEN,
        || unsafe { Th08.menu_takes_a_press() },
    );
    game
}

/// The screen that is up and the item under its cursor, as orb reads the pair.
fn where_it_is() -> (Option<i32>, Option<i32>) {
    unsafe { (Th08.front_end_screen(), Th08.front_end_cursor()) }
}

/// One decide, and then frames until the screen is the one wanted.
fn decides_into(game: &Fake, screen: Screen) {
    game.press(DECIDE);
    let wanted = screen as i32;
    game.frames_until(
        &format!("screen {wanted} came up"),
        FRAMES_FOR_A_SCREEN,
        || (unsafe { Th08.front_end_screen() }) == Some(wanted),
    );
}

/// **The three screens walked, and the run the last of them starts.**
#[test]
fn a_decide_at_each_screen_of_th08s_front_end_walks_into_a_run() {
    in_its_own_process(|| {
        let game = at_its_title("th08-the-front-end-walked");

        // The title menu, on the item a run is behind, which is where every launch of the real game reads
        // the cursor before anything has been left: `front end: screen 0, cursor on item 0`.
        assert_eq!(
            where_it_is(),
            (Some(Screen::Title as i32), Some(TITLE_ITEM_START)),
            "the launch did not come up at the title menu with the cursor on item 0",
        );
        // And a press taken, which is the gate at 0x467cac: the screen has been up long enough and is in
        // the phase its items are read in.
        assert!(
            unsafe { Th08.menu_takes_a_press() },
            "the title menu is up and orb reads it as taking no press",
        );

        // **And orb's own question answered before the walk goes anywhere**, which is what
        // `Th08::rewinds` answering true puts over this item: the first decide at a run is held back and
        // asked about, so a walk that pressed on would be pressing at a question and not at the game.
        // Which is the same step every recipe of this game now carries — see
        // `recipes/th08/a_run_started_and_given_up.yaml`.
        let log = game.log();
        game.press_until(DECIDE, "the mode question up", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        game.press_until(DECIDE, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });

        decides_into(&game, Screen::Difficulty);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Difficulty as i32), Some(DIFFICULTY_NORMAL)),
            "the difficulty select did not come up on Normal",
        );
        // **And no press held back over it**, which is `Th08::menu_takes_a_press` answering false for
        // every screen but the title: what a press does on this one has not been read, so orb holds none.
        assert!(
            !unsafe { Th08.menu_takes_a_press() },
            "orb holds a press back over a screen of this game it has not read",
        );

        decides_into(&game, Screen::Team);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Team as i32), Some(0)),
            "the team select did not come up with its cursor on the first team",
        );

        // A press as the team select comes up stays on the screen, and one after it has come up asks for
        // the run. Which is what `Th08::run_chosen` reads — the front end's scene with the run's wanted.
        game.press(DECIDE);
        assert_eq!(
            unsafe { Th08.front_end_screen() },
            Some(Screen::Team as i32),
            "a press as the team select came up left it",
        );
        for _ in 0..THE_TEAM_SELECT_COMES_UP_OVER {
            game.frame();
        }
        game.press(DECIDE);
        game.frames_until("the run started", FRAMES_FOR_A_SCREEN, || {
            game.state().in_run
        });
        // And the stage built, a frame on: this game builds a stage on a thread of its supervisor's, and the
        // numbers read below are what that thread writes — see `Th08::a_stage_has_just_been_built`.
        game.frames_until("the stage built", FRAMES_FOR_A_SCREEN, || {
            !game.image().the_loader_is_running()
        });

        let run = game.state();
        assert_eq!(
            (run.scene, run.wanted),
            (Scene::Run as i32, Scene::Run as i32),
            "the scene the walk ended in is not the one a run is played in",
        );
        assert_eq!(
            run.difficulty, DIFFICULTY_NORMAL,
            "the run started is not on the difficulty the select was left on",
        );
        // **And nothing of a screen read in the run, off a pointer that still names the front end.**
        //
        // This game writes `G_FRONT_END` once in the whole exe and never clears it, so a run's build
        // frees the object and the static goes on pointing at it — which is what orb read
        // `front end: screen 1900876895, cursor on item 0` off, on every frame of stage 1 of a measured
        // launch. So the pointer is still there below and the answer is still nothing: what declines is
        // the scene, and a filter on the pointer would answer that garbage instead.
        assert!(
            game.image().the_front_end_is_still_pointed_at(),
            "the laid-out run cleared a pointer the real game never clears, which is the one state \
             this asserts about",
        );
        assert_eq!(
            where_it_is(),
            (None, None),
            "orb read a screen and a cursor off the front end a run's build had freed",
        );
        // And the run's own numbers there, which is the pointer that build allocates.
        assert!(
            run.lives > 0 && run.bombs > 0,
            "the run started with none of the numbers its front end puts in place: {run}",
        );
    });
}

/// **And a direction moves the cursor**, which is what a recipe's `until: { cursor: 0 }` is answered off.
#[test]
fn a_direction_at_th08s_title_menu_moves_the_cursor_orb_reads() {
    in_its_own_process(|| {
        let game = at_its_title("th08-the-cursor-moved");

        // **To the stage practice and not the Extra**, a title with nothing cleared leaving out the two
        // items between — which a launch of the real game read going 0, 3, 4 … 8.
        game.press(DOWN);
        assert_eq!(
            unsafe { Th08.front_end_cursor() },
            Some(TITLE_ITEM_PRACTICE),
            "a press of down did not move the cursor orb reads past the two items nothing has lit",
        );

        // **And the screen it is on with it**, which is the other half of the pair: a cursor read off a
        // screen the walk had left would be a recipe's `until: { cursor: 0 }` answered about the wrong
        // list. The `front end: screen N, cursor on item M` line those steps are written against is
        // written only while a recipe is playing, so it is read where one is —
        // `the_cases_of_th08_the_real_game_is_tested_with`.
        assert_eq!(
            unsafe { Th08.front_end_screen() },
            Some(Screen::Title as i32),
            "a press of down left the title menu",
        );
    });
}

/// **And nothing read off it on the frame the run is asked for**, which is the scene the front end is on
/// with the run's own wanted beside it.
///
/// **Two launches took two gates to settle.** The first read `front end: screen 1900876895, cursor on
/// item 0` on every frame of a stage, through a filter that only caught the zero `G_FRONT_END` holds
/// before its first register; gated on the scene *now*, the second still read `front end: screen -1,
/// cursor on item 26280128` — once, and the two halves disagree because the freed object's head page is
/// still committed where its tail is not, so `read_committed` answers the cursor and declines the
/// screen. So the gate is the settled pair: the scene the front end is on **and** nothing else asked
/// for.
#[test]
fn nothing_of_th08s_front_end_is_read_on_the_frame_a_run_is_asked_for() {
    in_its_own_process(|| {
        let game = at_its_title("th08-the-front-end-asked-past");

        // The front end up and settled, which is where a screen is worth reading.
        assert!(
            unsafe { Th08.front_end_screen() }.is_some(),
            "the title menu is up and orb reads no screen of it",
        );

        // And the run asked for: the scene is still the front end's and the wanted is the run's, which
        // is the one frame the game's own transition leaves in that pair.
        game.image().is_in_scene(Scene::FrontEnd, Scene::Run);
        assert!(
            unsafe { Th08.run_chosen() },
            "the pair this asserts about is not the one orb reads a run being chosen from",
        );
        assert_eq!(
            where_it_is(),
            (None, None),
            "orb read a screen or a cursor on the frame the run was asked for, which is the frame the \
             front end goes on",
        );
    });
}

/// The character the team select is left on, which is one of the twelve and not the first — so a run
/// read at cursor 0 cannot pass for it.
const A_CHARACTER: i32 = 7;
/// And the difficulty the screen before it settled, which is Hard rather than the Normal a run this file
/// walks into gets.
const A_DIFFICULTY: u8 = 2;

/// **The run the team select points at, which is the mark that says a run of it was left unfinished** —
/// and the one screen of this game that knows: the cursor there *is* the character, and the difficulty
/// is the byte the screen before wrote.
#[test]
fn the_run_th08s_team_select_points_at_is_the_cursor_and_the_difficulty_already_chosen() {
    in_its_own_process(|| {
        let game = at_its_title("th08-the-run-pointed-at");

        // Nothing at the title, which is a screen that knows neither.
        assert!(
            unsafe { Th08.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at on the title menu",
        );

        game.image().the_difficulty_chosen_is(A_DIFFICULTY);
        game.image().the_front_end_shows(Screen::Team, A_CHARACTER);
        game.frame();

        let pointed = unsafe { Th08.run_pointed_at() }.expect("the team select knows which run");
        assert_eq!(
            (pointed.difficulty, pointed.character, pointed.shot_type),
            (i32::from(A_DIFFICULTY), A_CHARACTER, 0),
            "the run the team select points at is not the cursor and the difficulty before it",
        );
        assert!(
            !pointed.practice,
            "the run a whole-run flow points at reads as a practice",
        );

        // And nothing again on the difficulty select, where the character has not been chosen.
        game.image()
            .the_front_end_shows(Screen::Difficulty, A_CHARACTER);
        assert!(
            unsafe { Th08.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at a screen before the character is chosen",
        );
    });
}
