//! One レガシーモード run of 妖々夢, which is the same game with none of orb's own work happening in it.
//!
//! `a_pointdevice_run_of_th07.rs` is what the mode *does*; this is what answering the other way costs, and it is
//! worth an e2e test of its own because every one of those things is something orb has to **not** do. A run whose
//! chapters were kept anyway would look exactly like this one until somebody died.
//!
//! The same laid-out game, driven the same way: it presses keys and reads back the game's own memory, the game's
//! own record of the card, and what orb put in the log.

use crate::fake::th07::{Fake, LIVES};
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::Th07;
use orb_core::game::{Game, Menu};
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::{Mode, aside};
use orb_sim::{Quad, keys};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the fight puts up, by its own record, named the way 妖々夢 names cards, and the shot this run is
/// played with — whose own word of that record the game counts its attempt into.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";
const SHOT: usize = 0;

/// How far into the stage the card is put up, and how far past it the run is played: past where 完全無欠モード
/// would have taken chapters, which is what makes their absence something this can read.
const CARD_AT: u32 = 240;
const PAST_IT: u32 = CARD_AT + 240;

/// The row the count of lives is in, as a quad to hold what was drawn against: `Th07::lives_row`'s own rectangle,
/// which is where the game paints that count and nothing of orb's may paint over in *this* mode.
fn lives_row() -> Quad {
    let row = Th07.lives_row();
    Quad {
        x: row.left,
        y: row.top,
        width: row.width,
        height: row.height,
        color: 0,
        texture: 0,
    }
}

/// **The question read off the screen as it is answered the other way**: the two modes, the cursor on the one
/// this run is going to be, and the lines under them saying what *that* one means.
///
/// Its own e2e test because it is the one thing about answering the other way that happens before the run: the
/// walk below is `Fake::in_a_legacy_run`, which answers it in one line and reads nothing of it.
#[test]
fn the_mode_question_over_th07_answered_the_other_way_is_read_off_the_screen() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-legacy-the-question",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        // The same question over the same title menu, with one press down the list of two — after the frames the
        // question reads nothing over, a direction being a key that cannot be pressed again for nothing.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        game.press_until(keys::Z, "the mode question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        // The cursor is on the other one now, which is a colour on the screen and not a number anywhere:
        // `menu_ui` draws the item under it in `SELECTED`. And the lines under the items say what *this* choice
        // means, which is not what the other one's said.
        // A frame that drew, both lines read off that same one: a frame orb was late for draws nothing at all —
        // see `Launched::a_frame_that_says`.
        let pointdevice = game.a_frame_that_says(Mode::Pointdevice.name(LANGUAGE));
        let legacy = game.says(Mode::Normal.name(LANGUAGE));
        assert_eq!(
            (pointdevice.len(), legacy.len()),
            (1, 1),
            "the question is not the two modes:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            legacy[0].color, SELECTED,
            "the cursor is not on レガシーモード",
        );
        assert_eq!(pointdevice[0].color, NORMAL);
        assert_eq!(
            game.says(aside(Menu::Run, Mode::Normal, LANGUAGE)[0]).len(),
            1,
            "what レガシーモード means is not said under it",
        );
        assert!(
            game.says(aside(Menu::Run, Mode::Pointdevice, LANGUAGE)[0])
                .is_empty(),
            "the lines under the other choice are still on the screen",
        );
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the other mode was not chosen:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

#[test]
fn a_legacy_run_of_th07_keeps_no_chapters_offers_no_retry_and_leaves_nothing_behind() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-legacy",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);

        // ── 1. レガシーモード, and the stage it starts. The one thing orb does to the run is write down that it
        // is doing nothing: the hook over the moment a stage's numbers are put in place is still in the path,
        // and it keeps nothing.
        game.in_a_legacy_run();
        game.one_frame_to_drain_the_log();
        assert!(
            log.said("resume: stage 1 of a run orb is not keeping; nothing of it is written down"),
            "the run was written down after all:\n  {}",
            log.lines().join("\n  ")
        );

        // ── 2. The stage is played past where 完全無欠モード's own chapters were, and none of them is here: no
        // snapshot is taken, so there is nothing to go back to and nothing to mark the lives with.
        //
        // What the question drew on the way in is forgotten first: that covers the whole output, the count of
        // lives with it, and what is being asked about here is the frames of the stage.
        game.forget();
        game.frames_until("the frames before the card", CARD_AT + 8, || {
            game.state().stage_frames >= CARD_AT
        });
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until(
            "the stage past where a card would have been a chapter",
            600,
            || game.state().stage_frames > PAST_IT,
        );
        assert!(
            !log.said("chapter 1 (stage start)"),
            "a chapter was taken in a run that cannot be rewound:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("lives: the brush is"),
            "the lives were marked as disabled in a run that loses them:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !game
                .drawn()
                .quads
                .iter()
                .any(|quad| quad.overlaps(&lives_row())),
            "something was drawn over the count of lives",
        );
        assert!(
            game.says("DISABLE").is_empty(),
            "the lives were called disabled in a run that loses them",
        );
        // The game counted its own attempt at the card where the card started, which is what it does in either
        // mode: that number is the game's and orb only ever adds to it.
        assert_eq!(game.image().card_attempts(CARD, SHOT), 1);

        // ── 3. 被弾. A life, and no menu: the game takes the death and the run carries on.
        let before = game.state();
        game.dies();
        game.one_frame();
        assert_eq!(
            game.state().lives,
            before.lives - 1,
            "the death cost a life",
        );
        assert_eq!(game.state().deaths, 1, "and is counted against the run");
        assert!(
            !log.said("died in chapter"),
            "the retry menu was offered in a run with nothing to retry:\n  {}",
            log.lines().join("\n  ")
        );
        let died_at = game.state().stage_frames;
        game.frames(10);
        assert!(
            game.state().stage_frames > died_at,
            "the game stopped updating, which is what a retry menu would do to it",
        );
        assert_eq!(
            game.image().card_attempts(CARD, SHOT),
            1,
            "an attempt was counted for a death that cost a life",
        );
        // And the lives it lost are the run's own: a mode that keeps nothing takes them for good, which is what
        // the mark over the count is *for* in the other one.
        assert_eq!(game.state().lives, LIVES as i8 - 1);

        // ── 4. And nothing is left behind for a later launch to offer: the run goes on past where the other
        // mode's chapter was written down, and no file appears.
        game.frames_until_a_thread(
            "the frames a chapter would have been written down in",
            || log.said("resume: "),
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a run that keeps nothing left a chapter behind: {:?}",
            orb_core::resume::left(game.dir()),
        );
    });
}
