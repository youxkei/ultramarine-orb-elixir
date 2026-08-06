//! 完全無欠モード, chosen at the question orb puts over the game's own title menu.
//!
//! What the question *decides* — which of the two a run is, and which of the two rankings a `Score`
//! is — driven the way somebody meets it: a 紅魔郷 at its title menu, keys pressed at it, and what
//! came of that read off the screen and out of orb's log. Not by calling `Question::update`, which is
//! how these were written before and is the shape `docs/adr/0001` rules out: a test that reaches in
//! and works the question itself is playing the game's part from a script of its own, and it cannot
//! fail for the reasons the game supplies — the press being held back, the screen underneath never
//! having seen it, the frames the game's own menu ignores a press for.
//!
//! **What is on the screen is how the answer is read.** The question draws the two modes with the one
//! under the cursor in `SELECTED`, so which would be chosen is a colour rather than a field, and
//! `Screen::says` is what turns a quad back into the text it drew. The mode itself is orb's own to
//! report, and it does: `mode: pointdevice, was normal`.
//!
//! Several games in one file, each dropped before the next is attached — which the recording device's
//! own lock enforces, one game at a time — because what a scenario needs is a launch and there is
//! nothing about a launch that outlives it.

mod fake;

use fake::th06::Fake;
use fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb::menu_ui::{NORMAL, SELECTED};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Screen, item};
use orb_core::game::{Menu, RunStart};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// The run every game here offers, none of which gets as far as playing one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// A game at its title menu, with the question not yet asked.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    game
}

/// The question put up, on the press that would have chosen the item under the cursor.
///
/// # Panics
/// If it is not on the screen afterwards, which is the only way anything below means anything.
fn ask(game: &Fake) {
    game.press(keys::Z);
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run)).is_empty(),
        "the press did not put the question up: {:?}",
        game.log().lines(),
    );
    // And past the frames it reads nothing over, which is what the press it went up on is held off
    // by — a direction pressed inside those is one nothing moved on. See `READS_KEYS_AFTER`.
    game.frames(READS_KEYS_AFTER);
}

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
///
/// # Panics
/// If the two are not both on the screen with exactly one of them lit, since a question showing
/// anything else is not one anybody could answer.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for (mode, text) in orb_core::mode::CHOICES {
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
    game.one_frame();
    !game.says(title(Menu::Run)).is_empty()
}

/// How many times orb has reported an outcome for this question — a mode chosen, or neither.
///
/// Counted rather than looked for, because a scenario asks the question more than once and a log line
/// once written is written for good: a condition that waits for one to *appear* is a condition already
/// true the second time round, which is a press nobody made and an answer nobody gave.
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

/// Back out of the game's own shot type select, to the title menu the question is asked over.
///
/// Pressed until it lands: the read after one of orb's questions comes down has nothing new in it
/// whatever is held — which is what `SETTLE_KEYS` asks for, so that a key held on the question is not
/// a fresh press to the screen underneath — and the first press here falls on exactly that read.
fn back_out(game: &Fake) {
    game.press_until(keys::X, "the shot type select left", || {
        game.image().front_end_now().screen == Screen::Title
    });
}

/// The cursor starts on the mode orb is already in, so 完全無欠モード is one press away from somebody
/// who played a pointdevice run last time — and the press that answers is handed to the game's own
/// menu, which is what starts the run.
///
/// Both keys the game's own menus decide with answer here, which is why this asks twice: the question
/// is over the game's screen, and a key that chooses an item there has to choose here.
#[test]
fn one_press_chooses_the_mode_the_cursor_is_on_and_the_run_starts() {
    in_its_own_process(|| {
        let game = game("mode-chosen");
        for key in [keys::Z, keys::RETURN] {
            game.at_the_title_menu();
            ask(&game);
            assert_eq!(
                under_the_cursor(&game),
                Mode::Pointdevice,
                "the cursor is not on the mode orb is in",
            );
            answer(&game, key);
            // The press orb held back is handed over, so the game's own menu chooses the item it was
            // asked about — which is the run, and this is the screen that answers what run it is.
            game.frames_until("the run being asked for", 90, || {
                game.image().front_end_now().screen == Screen::ShotType
            });
            assert!(
                game.log().said("mode: pointdevice, was pointdevice"),
                "the mode is not what was chosen: {:?}",
                game.log().lines(),
            );
            // Back out of it, so the second key is asked the same question the first was.
            back_out(&game);
        }
    });
}

/// And somebody in レガシーモード reaches 完全無欠 by moving the cursor once. Down or up, either of them:
/// with two items each direction is the other one.
///
/// Which is also what says the answer is remembered: the question is asked a second time in the same
/// game, and the cursor starts on what the first one chose.
#[test]
fn the_cursor_moves_either_way_and_the_mode_it_lands_on_is_kept() {
    in_its_own_process(|| {
        let game = game("mode-moved");
        ask(&game);
        // Down to レガシーモード and chosen there, which is the mode the next question starts on.
        game.press(keys::DOWN);
        assert_eq!(under_the_cursor(&game), Mode::Normal);
        answer(&game, keys::Z);
        assert!(
            game.log().said("mode: normal, was pointdevice"),
            "レガシーモード was not chosen: {:?}",
            game.log().lines(),
        );

        // Asked again, from the other end: the cursor is where the last answer left it, and up moves the
        // same one step the other way.
        game.frames_until("the run being asked for", 90, || {
            game.image().front_end_now().screen == Screen::ShotType
        });
        back_out(&game);
        game.at_the_title_menu();
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
            game.log().said("mode: pointdevice, was normal"),
            "完全無欠モード was not chosen the second time: {:?}",
            game.log().lines(),
        );
    });
}

/// Cancelling leaves neither chosen, on the bomb key the game's own menus read as back and on escape.
///
/// Which is not the same as choosing レガシーモード: the press that would have started a run was held
/// back, so a cancelled question is a run not started at all rather than one started without chapters.
/// The screen underneath is where it was — the title menu, on the item the question was about.
#[test]
fn cancelling_starts_no_run_and_leaves_the_screen_where_it_was() {
    in_its_own_process(|| {
        let game = game("mode-cancelled");
        for key in [keys::X, keys::ESCAPE] {
            game.at_the_title_menu();
            ask(&game);
            answer(&game, key);
            assert!(
                game.log().said("mode: not chosen on the keyboard"),
                "{key:#04x} did not cancel: {:?}",
                game.log().lines(),
            );
            assert!(
                !still_asking(&game),
                "the question is still up after being cancelled: {:?}",
                game.log().lines(),
            );
            assert!(
                !game.log().said("mode: answered on the"),
                "cancelling answered the question: {:?}",
                game.log().lines(),
            );
            // The game's own menu is where it was, on the item it was asked about, and no run was started.
            let front = game.image().front_end_now();
            assert_eq!(front.screen, Screen::Title, "the screen underneath moved");
            assert_eq!(front.cursor, item::GAME_START);
            assert!(!game.state().in_run, "a run was started by a cancel");
        }
    });
}

/// Nothing is answered while the question holds its keys off, and the key that would answer is down
/// the whole time: the press it went up on is still held, being the press that was kept from the game.
///
/// Then a key held down answers once and not once a frame — a question that answered every frame
/// would choose, and choose again, and the second answer would land on a run already started.
#[test]
fn the_press_it_went_up_on_does_not_answer_it_and_a_held_key_answers_once() {
    in_its_own_process(|| {
        let game = game("mode-grace");
        // Held from before the question and never let go: the press that put it up.
        game.keyboard().set(keys::Z, true);
        game.frames_until("the question", 8, || {
            !game.says(title(Menu::Run)).is_empty()
        });
        for _ in 0..90 {
            game.frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a key held from before the question answered it: {:?}",
                game.log().lines(),
            );
        }
        assert!(still_asking(&game), "the question came down by itself");

        // Let it up and press it again, which is what somebody actually does. One answer, and then
        // nothing more however long it is held.
        game.keyboard().set(keys::Z, false);
        game.frame();
        game.keyboard().set(keys::Z, true);
        game.frames_until("the mode chosen", 8, || outcomes(&game) == 1);
        let once = game.log().lines().len();
        for _ in 0..30 {
            game.frame();
        }
        assert!(
            !game.log().lines()[once..]
                .iter()
                .any(|line| line.contains("mode: answered on the")),
            "the question answered again with the key still down: {:?}",
            &game.log().lines()[once..],
        );
    });
}

/// Alt-tabbed away, nothing is pressed however hard: typing in another window must not choose a mode,
/// and a key held while the game goes to the back must not be a press when it comes forward.
///
/// And a host that will not say what is down at all reads as nothing down rather than as whatever was
/// down last — `GetKeyboardState` answers zero on a thread with no message queue, and taking the array
/// anyway would leave the last read standing, which on a menu that acts on edges is a key stuck down.
#[test]
fn keys_are_not_read_with_another_window_in_front_or_a_host_that_will_not_say() {
    in_its_own_process(|| {
        let game = game("mode-unread");
        ask(&game);

        game.sim().display().set_foreground(orb_api::Hwnd(0x9999));
        game.keyboard().set(keys::Z, true);
        for _ in 0..30 {
            game.frame();
        }
        assert_eq!(
            outcomes(&game),
            0,
            "the question was answered with another window in front",
        );

        // Back in front with the key still down. That is not a press — the edge happened while orb was
        // reading nothing — so the question is still up.
        game.sim().display().set_foreground(fake::WINDOW);
        for _ in 0..30 {
            game.frame();
        }
        assert_eq!(
            outcomes(&game),
            0,
            "a key held through an alt-tab answered the question on the way back",
        );

        // And with the host refusing, the same key pressed afresh reaches nothing.
        game.keyboard().set(keys::Z, false);
        game.frame();
        game.keyboard().refuse(true);
        game.keyboard().set(keys::Z, true);
        for _ in 0..30 {
            game.frame();
        }
        assert_eq!(
            outcomes(&game),
            0,
            "the question was answered while the host refused to say what was down",
        );
        assert!(still_asking(&game));

        // The host answering again is where the same press finally lands, which is what says the question
        // was reading all along and had nothing to read.
        game.keyboard().refuse(false);
        game.keyboard().set(keys::Z, false);
        game.frame();
        answer(&game, keys::Z);
    });
}

/// The ranking is asked the same way and with the same two modes: one choice for both, because the
/// ranking of pointdevice runs and a pointdevice run are the same file.
///
/// A different question, and it says so — `どちらのスコアを見る` where a run gets `モードを選ぶ` — and the
/// screen it lets through is the ranking rather than a run.
#[test]
fn the_ranking_is_asked_the_same_way_and_says_it_is_about_a_ranking() {
    in_its_own_process(|| {
        let game = game("mode-ranking");
        for _ in 0..item::SCORE {
            game.press(keys::DOWN);
        }
        assert_eq!(game.image().front_end_now().cursor, item::SCORE);
        game.press(keys::Z);
        game.one_frame();
        assert_eq!(
            game.says(title(Menu::Scores)).len(),
            1,
            "the question over a ranking is not the one asked about a ranking: {:?}",
            game.log().lines(),
        );
        assert!(
            game.says(title(Menu::Run)).is_empty(),
            "the question over a ranking is the one asked about a run",
        );
        assert_eq!(under_the_cursor(&game), Mode::Pointdevice);
        game.press_until(keys::Z, "完全無欠のスコア画面", || {
            game.image().scene() == orb_core::game::th06::image::Scene::Ranking
        });
    });
}
