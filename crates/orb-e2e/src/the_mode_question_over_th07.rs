//! Pointdevice mode, chosen at the question orb puts over th07's own title menu.
//!
//! What the question *decides* — which of the two a run is, and which of the two rankings a score file is — driven
//! the way somebody meets it: the game at its title menu, keys pressed at it, and what came of that read off the
//! screen and out of orb's log. Not by calling `Question::update`, which is the shape `docs/adr/0001` rules out.
//!
//! **What is on the screen is how the answer is read.** The question draws the two modes with the one under the
//! cursor in `SELECTED`, so which would be chosen is a colour rather than a field — and here it is drawn in a face
//! the machine lent, this game shipping no `font.ttf`.
//!
//! And what says the press was handed over is this game's own read: orb holds the decide back for the question and
//! feeds it on the read after an answer, so the word th07's own input read answers is where a press that was
//! kept and then given back shows up.

use crate::fake::th07::Fake;
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of th07's own input word the shot button is, which is the press orb holds back and hands over — see
/// `th07.rs`'s own pad test, which reads the same bit.
const SHOOT: u16 = 0x1;

/// A launch with the game at its title menu, past the frames its own screen ignores a press for and with an
/// overlay for orb to draw a question with.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.frames_until("an overlay", 8, || log.said("overlay: ready"));
    at_the_title_menu(&game);
    game
}

/// The title menu up with the cursor on the item that starts a run, which is the item orb has a question about.
///
/// And a frame of it, because orb decides whether to hold a press back at the *end* of an update: the first frame
/// the cursor is on an item it asks about is a frame whose press goes to the game, and what an e2e test about the
/// question wants is the frame after that.
fn at_the_title_menu(game: &Fake) {
    // A frame before the title is put up, where a question was answered before this: the press orb handed on
    // for that answer lands on the read after it, and this game's title acts on one — Start's flow walked. Landing
    // on the frame before the title is put back, it walks nothing.
    game.frame();
    game.image().the_front_end_is_up();
    game.image().the_title_menu_points_at_a_run();
    game.image().the_screen_has_been_up_for(60);
    game.one_frame();
    game.one_frame_to_drain_the_log();
    // And orb saying what it read of it, which is the line a launch is read back through: **no launch of
    // th07 has ever put this question up**, every e2e test here passes, and every constant on the way to it
    // has been held against the exe and matches — so what is left is a runtime fact, and this is the line
    // that will name it. See `Th07::says_what_it_reads_of_the_front_end`.
    assert!(
        game.log().said("front end: object 0x"),
        "orb did not say what it read of the front end:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// The question put up, on the press that would have chosen the item under the cursor.
///
/// # Panics
/// If it is not on the screen afterwards, which is the only way anything below means anything.
fn ask(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.press_until(keys::Z, "the question", || {
        log.said_since(from, "menu: Run is under the cursor, asking which mode")
    });
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run, LANGUAGE)).is_empty(),
        "the press did not put the question up:\n  {}",
        log.lines().join("\n  ")
    );
    // And past the frames it reads nothing over, which is what the press it went up on is held off by — a
    // direction pressed inside those is one nothing moved on. See `READS_KEYS_AFTER`.
    game.frames(READS_KEYS_AFTER);
}

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
///
/// # Panics
/// If the two are not both on the screen with exactly one of them lit, since a question showing anything else is
/// not one anybody could answer.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for mode in orb_core::mode::CHOICES {
        let text = mode.name(LANGUAGE);
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        } else {
            assert_eq!(drawn[0].color, NORMAL, "{text} is neither lit nor not");
        }
    }
    on.expect("neither mode is lit")
}

/// Whether the question is still on the screen, which is what "nothing was answered" looks like.
fn still_asking(game: &Fake) -> bool {
    // A frame that drew, and not the next one: a frame orb was late for draws nothing, and reading one of those
    // would be this answering that the question had been answered — see `Launched::a_frame_that_says`.
    !game
        .a_frame_that_says(title(Menu::Run, LANGUAGE))
        .is_empty()
}

/// How many times orb has reported an outcome for this question — a mode chosen, or neither.
///
/// Counted rather than looked for, because these ask more than once and a log line once written is written for
/// good: a condition that waits for one to *appear* is a condition already true the second time round.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// Answers it, whichever way `key` answers, and says nothing about which way that was.
fn answer(game: &Fake, key: u8) {
    let before = outcomes(game);
    game.press_until(key, "the question answered", || outcomes(game) > before);
}

/// The cursor starts on the mode orb is already in, so one press answers — and the press it answers with is the
/// one orb held back, which the game's own read gets on the frame after.
///
/// Both keys the game's own menus decide with answer here, which is why this asks twice: the question is over the
/// game's screen, and a key that chooses an item there has to choose here.
#[test]
fn one_press_chooses_the_mode_the_cursor_is_on_and_th07s_own_read_gets_it() {
    in_its_own_process(|| {
        let game = game("th07-mode-chosen");
        let log = game.log();
        for key in [keys::Z, keys::RETURN] {
            at_the_title_menu(&game);
            ask(&game);
            assert_eq!(
                under_the_cursor(&game),
                Mode::Pointdevice,
                "the cursor is not on the mode orb is in",
            );
            answer(&game, key);
            assert!(
                log.said("mode: pointdevice, was pointdevice"),
                "the mode is not what was chosen:\n  {}",
                log.lines().join("\n  ")
            );
            // The press orb held back, handed to the game's own read: what the game does with it is choose the
            // item it was asked about, which is the run.
            game.frames_until("the press handed to the game's own read", 8, || {
                game.word_the_input_read_answered() & SHOOT != 0
            });
        }
    });
}

/// And somebody in legacy mode reaches Pointdevice mode by moving the cursor once. Down or up, either of
/// them: with two items each direction is the other one.
///
/// Which is also what says the answer is remembered: the question is asked a second time in the same launch, and
/// the cursor starts on what the first one chose.
#[test]
fn the_cursor_of_th07s_question_moves_either_way_and_the_mode_it_lands_on_is_kept() {
    in_its_own_process(|| {
        let game = game("th07-mode-moved");
        let log = game.log();
        ask(&game);
        // Down to legacy mode and chosen there, which is the mode the next question starts on.
        game.press(keys::DOWN);
        assert_eq!(under_the_cursor(&game), Mode::Normal);
        answer(&game, keys::Z);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "レガシーモード was not chosen:\n  {}",
            log.lines().join("\n  ")
        );

        // Asked again, from the other end: the cursor is where the last answer left it, and up moves the same one
        // step the other way.
        at_the_title_menu(&game);
        ask(&game);
        assert_eq!(
            under_the_cursor(&game),
            Mode::Normal,
            "the question does not start on the mode the last one chose",
        );
        game.press(keys::UP);
        assert_eq!(under_the_cursor(&game), Mode::Pointdevice);
        answer(&game, keys::Z);
        assert!(
            log.said("mode: pointdevice, was normal"),
            "完全無欠モード was not chosen the second time:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// Cancelling leaves neither chosen, on the bomb key the game's own menus read as back and on escape.
///
/// Which is not the same as choosing legacy mode: the press that would have started a run was held back, so a
/// cancelled question is a run not started at all rather than one started without chapters. The screen underneath
/// is where it was — the title menu, on the item the question was about.
#[test]
fn cancelling_th07s_question_starts_no_run_and_leaves_the_screen_where_it_was() {
    in_its_own_process(|| {
        let game = game("th07-mode-cancelled");
        let log = game.log();
        for key in [keys::X, keys::ESCAPE] {
            at_the_title_menu(&game);
            let item = game.image().the_item_the_title_menu_points_at();
            ask(&game);
            answer(&game, key);
            assert!(
                log.said("mode: not chosen on the keyboard"),
                "{key:#04x} did not cancel:\n  {}",
                log.lines().join("\n  ")
            );
            assert!(
                !still_asking(&game),
                "the question is still up after being cancelled:\n  {}",
                log.lines().join("\n  ")
            );
            assert!(
                !log.said("mode: answered on the"),
                "cancelling answered the question:\n  {}",
                log.lines().join("\n  ")
            );
            // The game's own menu is where it was, on the item it was asked about, and no run was started.
            assert_eq!(
                game.image().the_screen_the_front_end_is_on(),
                game.image().screen_of_the_title_menu(),
                "the screen underneath moved",
            );
            assert_eq!(game.image().the_item_the_title_menu_points_at(), item);
            assert!(!game.state().in_run, "a run was started by a cancel");
        }
    });
}

/// Nothing is answered while the question holds its keys off, and the key that would answer is down the whole
/// time: the press it went up on is still held, being the press that was kept from the game.
///
/// Then a key held down answers once and not once a frame — a question that answered every frame would choose,
/// and choose again, and the second answer would land on a run already started.
#[test]
fn the_press_th07s_question_went_up_on_does_not_answer_it_and_a_held_key_answers_once() {
    in_its_own_process(|| {
        let game = game("th07-mode-grace");
        let log = game.log();
        // Held from before the question and never let go: the press that put it up.
        game.keyboard().set(keys::Z, true);
        game.frames_until("the question", 8, || {
            !game.says(title(Menu::Run, LANGUAGE)).is_empty()
        });
        for _ in 0..90 {
            game.one_frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a key held from before the question answered it:\n  {}",
                log.lines().join("\n  ")
            );
        }
        assert!(still_asking(&game), "the question came down by itself");

        // Let it up and press it again, which is what somebody actually does. One answer, and then nothing more
        // however long it is held.
        game.keyboard().set(keys::Z, false);
        game.one_frame();
        game.keyboard().set(keys::Z, true);
        game.frames_until("the mode chosen", 8, || outcomes(&game) == 1);
        let once = log.written();
        game.frames(30);
        assert!(
            !log.said_since(once, "mode: answered on the"),
            "the question answered again with the key still down:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// Alt-tabbed away, nothing is pressed however hard: typing in another window must not choose a mode, and a key
/// held while the game goes to the back must not be a press when it comes forward.
///
/// And a host that will not say what is down at all reads as nothing down rather than as whatever was down last —
/// `GetKeyboardState` answers zero on a thread with no message queue, and taking the array anyway would leave the
/// last read standing, which on a menu that acts on edges is a key stuck down.
#[test]
fn keys_are_not_read_at_th07s_question_with_another_window_in_front() {
    in_its_own_process(|| {
        let game = game("th07-mode-unread");
        let log = game.log();
        ask(&game);

        // Another program in front, with the answering key held down the whole time.
        game.sim()
            .display()
            .set_foreground(orb_api::Hwnd(crate::fake::WINDOW.0 + 1));
        game.keyboard().set(keys::Z, true);
        game.frames(60);
        assert_eq!(
            outcomes(&game),
            0,
            "a key held into another program's window answered the question:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(still_asking(&game), "the question came down by itself");

        // And the game's window in front again with that key still down: it is not a fresh press, so nothing is
        // answered by the hand that never moved.
        game.sim().display().set_foreground(crate::fake::WINDOW);
        game.frames(30);
        assert_eq!(
            outcomes(&game),
            0,
            "the key held while the window was behind answered as it came forward:\n  {}",
            log.lines().join("\n  ")
        );

        // A host that says nothing about the keyboard reads as nothing held, so the key that was down is not one
        // the question sees either.
        game.keyboard().refuse(true);
        game.frames(30);
        assert_eq!(
            outcomes(&game),
            0,
            "a host that would not say what was down answered the question:\n  {}",
            log.lines().join("\n  ")
        );
        game.keyboard().refuse(false);

        // And it is still answerable, which is what says none of the above was the question breaking: let the key
        // up and press it again.
        game.keyboard().set(keys::Z, false);
        game.one_frame();
        answer(&game, keys::Z);
    });
}
